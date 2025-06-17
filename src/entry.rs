use std::{
    fs::write,
    io::{Result, Write},
    path::Path,
    rc::Rc,
};

use hex::ToHex;
use sha2::{Digest, Sha256};

/// Entry in the General Ledger.
#[derive(Debug)]
pub enum Entry {
    Entry {
        name: String,
        description: String,
        lines: Vec<EntryLine>,
        previous_entry: Rc<Entry>,
    },
    Origin {
        year: usize,
    },
}

impl Entry {
    pub(crate) fn create_new(
        location: &Path,
        name: impl Into<String>,
        description: impl Into<String>,
        lines: Vec<EntryLine>,
        previous_entry: Rc<Entry>,
    ) -> Result<Entry> {
        let entry = Entry::Entry {
            name: name.into(),
            description: description.into(),
            lines,
            previous_entry,
        };
        let mut buffer: Vec<u8> = vec![];
        entry
            .serialize(&mut buffer)
            .expect("Should not break the stack");
        let hash: String = {
            let mut hasher = Sha256::new();
            hasher.update(&buffer);
            hasher.finalize().encode_hex()
        };
        let path = location.join(hash);
        write(path, buffer)?;
        Ok(entry)
    }

    fn get_hash_hex(&self) -> String {
        let mut buffer: Vec<u8> = vec![];
        self.serialize(&mut buffer).expect("This will not fail");
        let mut hasher = Sha256::new();
        hasher.update(&buffer);
        hasher.finalize().encode_hex()
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
    fn serialize<W: Write>(&self, mut output: W) -> Result<()> {
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
                let previous_hex = previous_entry.get_hash_hex();
                output.write_all(&previous_hex.into_bytes())?;
            }
        }
        output.flush()?;
        Ok(())
    }
}

/// Journal EntryLine used for accounting
#[derive(Debug)]
pub struct EntryLine {
    account: String,
    amount: usize, // Amount in smallest currency unit
    side: Side,    // true for debit, false for credit
    description: Option<String>,
}

#[derive(Debug, PartialEq)]
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
}
