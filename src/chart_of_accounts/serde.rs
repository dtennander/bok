use std::io::{Read, Result, Write};

use crate::chart_of_accounts::{Account, AccountType};
use crate::read::read;

impl Account {
    /// Serialize an Account into binary form
    ///
    /// Account Serialization Format:
    /// +--------------------+--------------+--------------+------------+
    /// | account_number (8B)| name_len (4B)| desc_len (4B)| type (1 B) |
    /// +--------------------+--------------+--------------+------------+
    /// | name data (variable)             | description data (variable)|
    /// +----------------------------------+----------------------------+
    pub(crate) fn serialize<W: Write>(&self, output: &mut W) -> Result<()> {
        // Write account_number (8 bytes, little-endian)
        output.write_all(&(self.account_number as u64).to_le_bytes())?;
        // Write name length (4 bytes, little-endian)
        output.write_all(&(self.name.len() as u32).to_le_bytes())?;
        // Write description length (4 bytes, little-endian)
        output.write_all(&(self.description.len() as u32).to_le_bytes())?;
        // Write account_type discriminant (1 byte)
        let type_byte: u8 = match self.account_type {
            AccountType::Asset => 0x00,
            AccountType::Liability => 0x01,
            AccountType::Equity => 0x02,
            AccountType::Revenue => 0x03,
            AccountType::Expense => 0x04,
        };
        output.write_all(&[type_byte])?;
        // Write name data
        output.write_all(self.name.as_bytes())?;
        // Write description data
        output.write_all(self.description.as_bytes())?;
        Ok(())
    }

    /// Deserialize an Account from binary form
    pub(crate) fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        let buffer: [u8; 8] = [0x00; 8];

        // Read account_number (8 bytes, little-endian)
        read!(account_number(u64) as usize from reader using buffer);

        // Read name_len and desc_len (4 bytes each)
        read!(name_len(u32) as usize from reader using buffer);
        read!(desc_len(u32) as usize from reader using buffer);

        // Read account_type discriminant (1 byte)
        read!(type_byte(u8) from reader using buffer);
        let account_type = match type_byte {
            0x00 => AccountType::Asset,
            0x01 => AccountType::Liability,
            0x02 => AccountType::Equity,
            0x03 => AccountType::Revenue,
            0x04 => AccountType::Expense,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unknown AccountType discriminant: {:#04x}", type_byte),
                ));
            }
        };

        // Read name and description
        read!(name(name_len) as String from reader);
        read!(description(desc_len) as String from reader);

        Ok(Account {
            account_number,
            name,
            description,
            account_type,
        })
    }
}
