use std::io::{Result, Write, empty};

use crate::{read::read, tee_writer::TeeWriter};
use hex::ToHex;
use sha2::{Digest, Sha256};

/// Entry in the General Ledger.
#[derive(Debug, Clone)]
pub enum Entry {
    Entry {
        name: String,
        description: String,
        lines: Vec<EntryLine>,
        previous_entry: String,
    },
    Origin {
        year: usize,
    },
}

impl Entry {
    pub(crate) fn create_new(
        name: impl Into<String>,
        description: impl Into<String>,
        lines: Vec<EntryLine>,
        previous_entry: String,
    ) -> Result<Entry> {
        let entry = Entry::Entry {
            name: name.into(),
            description: description.into(),
            lines,
            previous_entry,
        };
        Ok(entry)
    }

    pub(crate) fn get_hash_hex(&self) -> String {
        self.serialize(empty()).expect("This will not fail")
    }

    /// Serialize an entry into binary form
    ///
    /// Origin Variant (0x00):
    /// +--------+--------+--------+--------+--------+--------+--------+--------+
    /// |  0x00  |                      year (8 bytes)                          |
    /// +--------+--------+--------+--------+--------+--------+--------+--------+
    ///
    /// Entry Variant (0x01):
    /// +--------+--------+--------+--------+--------+--------+--------+--------+
    /// |  0x01  |      name_len (4 bytes)        |      desc_len (4 bytes)     |
    /// +--------+--------+--------+--------+--------+--------+--------+--------+
    /// |                    name data (variable length)                        |
    /// +--------+--------+--------+--------+--------+--------+--------+--------+
    /// |                 description data (variable length)                    |
    /// +--------+--------+--------+--------+--------+--------+--------+--------+
    /// |      lines_count (4 bytes)        |                                   |
    /// +--------+--------+--------+--------+                                   +
    /// |                    lines data (variable length)                       |
    /// +                                                                       +
    /// |                              ...                                      |
    /// +--------+--------+--------+--------+--------+--------+--------+--------+
    /// |                    previous_entry_id (32 bytes)                       |
    /// +                                                                       +
    /// |                              ...                                      |
    /// +                                                                       +
    /// |                                                                       |
    /// +--------+--------+--------+--------+--------+--------+--------+--------+
    ///
    pub(crate) fn serialize<W: Write>(&self, output: W) -> Result<String> {
        let mut output = TeeWriter::new(output, Sha256::new());

        match self {
            Entry::Origin { year } => {
                // Write discriminant for Origin
                output.write_all(&[0x00])?;
                // Write year as 8-byte little-endian
                output.write_all(&year.to_le_bytes())?;
            }

            Entry::Entry {
                name,
                description,
                lines,
                previous_entry,
            } => {
                // Write discriminant for Entry
                output.write_all(&[0x01])?;

                // Write name length and description length
                output.write_all(&(name.len() as u32).to_le_bytes())?;
                output.write_all(&(description.len() as u32).to_le_bytes())?;

                // Write name data
                output.write_all(name.as_bytes())?;

                // Write description data
                output.write_all(description.as_bytes())?;

                // Write lines count
                output.write_all(&(lines.len() as u32).to_le_bytes())?;

                // Write each EntryLine
                for line in lines {
                    line.serialize(&mut output)?;
                }

                // Write previous_entry_id (32 bytes) at the end
                output.write_all(&previous_entry.clone().into_bytes())?;
            }
        }
        output.flush()?;
        let (_, hash) = output.into_inner();
        Ok(hash.finalize().encode_hex())
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Empty byte array",
            ));
        }
        let mut cursor = 0;
        // Read discriminant
        let discriminant = bytes[0];
        cursor += 1;

        match discriminant {
            0x00 => {
                // Origin variant: need 8 more bytes for year
                read!(year(u64) as usize from bytes[cursor]);
                Ok(Entry::Origin { year })
            }
            0x01 => {
                read!(name_len(u32) as usize from bytes[cursor]);
                read!(desc_len(u32) as usize from bytes[cursor]);
                read!(name(name_len) as String from bytes[cursor]);
                read!(description(desc_len) as String from bytes[cursor]);
                read!(lines_count(u32) from bytes[cursor]);

                // Read lines
                let mut lines = Vec::new();
                for i in 0..lines_count {
                    let (line, consumed) =
                        EntryLine::from_bytes(&bytes[cursor..]).map_err(|e| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Failed to deserialize line {}: {}", i, e),
                            )
                        })?;
                    cursor += consumed;
                    lines.push(line);
                }
                read!(previous_entry(32) as String from bytes[cursor]);
                Ok(Entry::Entry {
                    name,
                    description,
                    lines,
                    previous_entry,
                })
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unknown discriminant: {:#04x}", discriminant),
            )),
        }
    }
}

/// Journal EntryLine used for accounting
#[derive(Debug, Clone)]
pub struct EntryLine {
    account: String,
    amount: usize, // Amount in smallest currency unit
    side: Side,    // true for debit, false for credit
    description: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Side {
    Debit,
    Credit,
}

impl EntryLine {
    /// Create new entry line
    pub fn new(
        account: impl Into<String>,
        amount: usize,
        side: Side,
        description: Option<impl Into<String>>,
    ) -> Self {
        Self {
            account: account.into(),
            amount,
            side,
            description: description.map(Into::into),
        }
    }

    /// Serialize an EntryLine into binary form
    ///
    /// Structure:
    /// +--------+--------+--------+--------+--------+--------+--------+--------+
    /// |      account_len (4 bytes)        |        amount (8 bytes)           |
    /// +--------+--------+--------+--------+--------+--------+--------+--------+
    /// |  side  | d_flag |                                                     |
    /// +--------+--------+                                                     +
    /// |                    account data (variable length)                     |
    /// +--------+--------+--------+--------+--------+--------+--------+--------+
    /// |                description_len (4 bytes, if d_flag = 0x01)            |
    /// +--------+--------+--------+--------+--------+--------+--------+--------+
    /// |                description data (variable length, if present)         |
    /// +--------+--------+--------+--------+--------+--------+--------+--------+
    fn serialize<W: Write>(&self, mut output: W) -> Result<()> {
        // Write account length (4 bytes)
        output.write_all(&(self.account.len() as u32).to_le_bytes())?;

        // Write amount (8 bytes)
        output.write_all(&(self.amount as u64).to_le_bytes())?;

        // Write side (1 byte: 0x00 = credit, 0x01 = debit)
        let side_byte = match self.side {
            Side::Credit => 0x00,
            Side::Debit => 0x01,
        };
        output.write_all(&[side_byte])?;

        // Write description flag (1 byte)
        let desc_flag = if self.description.is_some() {
            0x01
        } else {
            0x00
        };
        output.write_all(&[desc_flag])?;

        // Write account data
        output.write_all(self.account.as_bytes())?;

        // Write description if present
        if let Some(desc) = &self.description {
            output.write_all(&(desc.len() as u32).to_le_bytes())?;
            output.write_all(desc.as_bytes())?;
        }

        Ok(())
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> std::io::Result<(Self, usize)> {
        let mut cursor = 0;

        // Need at least 14 bytes for the fixed fields (account_len + amount + side + d_flag)
        if bytes.len() < 14 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Insufficient bytes for EntryLine header",
            ));
        }
        read!(account_len(u32) as usize from bytes[cursor]);
        read!(amount(u64) as usize from bytes[cursor]);
        // Read side (1 byte)
        let side_byte = bytes[cursor];
        let side = match side_byte {
            0x00 => Side::Credit,
            0x01 => Side::Debit,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid side byte: {:#04x}", side_byte),
                ));
            }
        };
        cursor += 1;

        // Read description flag (1 byte)
        let desc_flag = bytes[cursor];
        let has_description = match desc_flag {
            0x00 => false,
            0x01 => true,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid description flag: {:#04x}", desc_flag),
                ));
            }
        };
        cursor += 1;

        read!(account(account_len) as String from bytes[cursor]);

        // Read description if present
        let description = if has_description {
            read!(desc_len(u32) as usize from bytes[cursor]);
            read!(desc(desc_len) as String from bytes[cursor]);
            Some(desc)
        } else {
            None
        };

        let entry_line = EntryLine {
            account,
            amount,
            side,
            description,
        };

        Ok((entry_line, cursor))
    }
}
