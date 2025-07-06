use crate::read::read;
use std::io::{Read, Result, Write};

/// Journal EntryLine used for accounting
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryLine {
    pub account: String,
    pub amount: usize, // Amount in smallest currency unit
    pub side: Side,    // true for debit, false for credit
    pub description: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum Side {
    Debit,
    Credit,
}

impl EntryLine {
    /// Simple constructor for EntryLine
    pub fn new(account: &str, amount: usize, side: Side, description: Option<String>) -> Self {
        EntryLine {
            account: account.to_string(),
            amount,
            side,
            description,
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
    pub(crate) fn serialize<W: Write>(&self, mut output: W) -> Result<()> {
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

    pub(crate) fn deserialize<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let buffer: [u8; 8] = [0; 8];
        read!(account_len(u32) as usize from reader using buffer);
        read!(amount(u64) as usize from reader using buffer);
        read!(side_byte(u8) from reader using buffer);
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
        read!(desc_flag(u8) from reader using buffer);
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

        read!(account(account_len) as String from reader);

        // Read description if present
        let description = if has_description {
            read!(desc_len(u32) as usize from reader using buffer);
            read!(desc(desc_len) as String from reader);
            Some(desc)
        } else {
            None
        };

        Ok(EntryLine {
            account,
            amount,
            side,
            description,
        })
    }
}
