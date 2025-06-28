use std::io::{Result, Write};

use hex::ToHex;
use sha2::{Digest, Sha256};

use super::{Entry, EntryLine};
use crate::read::read;
use crate::tee_writer::TeeWriter;

impl Entry {
    /// Serialize an entry into binary form
    ///
    /// Returns the hash as the result if successful
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
                read!(year(u64) from bytes[cursor]);
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
