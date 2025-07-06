use std::io::{BufReader, Read, Result, Seek, Write, empty};

use chrono::{DateTime, Datelike, NaiveDate};
use flate2::Compression;
use flate2::read::GzDecoder;
use hex::ToHex;
use sha2::{Digest, Sha256};

use super::{Entry, EntryLine};
use crate::read::read;
use crate::tee_writer::TeeWriter;
use flate2::write::GzEncoder;

impl Entry {
    /// Serialize an entry into binary form
    ///
    /// Returns the hash as the result if successful
    ///
    /// Origin Variant (0x00):
    /// +--------+--------------------------+-------------------------+
    /// | 0x00   | year (8 bytes)           | timestamp (8 bytes)     |
    /// +--------+--------------------------+-------------------------+
    ///
    /// Entry Variant (0x01):
    /// +--------+------------+-----------------+----------------+----------------+
    /// | 0x01   | date (4 B) | timestamp (8 B) | name_len (4 B) | desc_len (4 B) |
    /// +--------+------------+-----------------+----------------+----------------+
    /// | name data (variable length)                                             |
    /// +-------------------------------------------------------------------------+
    /// | description data (variable length)                                      |
    /// +-------------------------------------------------------------------------+
    /// | lines_count (4 B)                                                       |
    /// +-------------------------------------------------------------------------+
    /// | lines data (variable length)                                            |
    /// +-------------------------------------------------------------------------+
    /// | previous_entry_id (32 B)                                                |
    /// +-------------------------------------------------------------------------+
    ///
    pub(crate) fn serialize<W: Write + Seek>(&self, output: W) -> Result<String> {
        let zipper = GzEncoder::new(output, Compression::default());
        let mut output = TeeWriter::new(zipper, Sha256::new());

        match self {
            Entry::Origin { timestamp, year } => {
                // Write discriminant for Origin
                output.write_all(&[0x00])?;
                // Write year as 8-byte little-endian
                output.write_all(&year.to_le_bytes())?;
                // Write timestamp as 8-byte little-endian
                let epoch_secs = timestamp.timestamp();
                output.write_all(&epoch_secs.to_le_bytes())?;
            }

            Entry::Entry {
                timestamp,
                event_date,
                name,
                description,
                lines,
                previous_entry,
            } => {
                // Write discriminant for Entry
                output.write_all(&[0x01])?;

                // Write event_date (as days since year 0, 4 bytes, little endian)
                let date_days = event_date.num_days_from_ce();
                output.write_all(&date_days.to_le_bytes())?;

                // Write timestamp (as seconds since epoch, 8 bytes, little endian)
                let epoch_secs = timestamp.timestamp();
                output.write_all(&epoch_secs.to_le_bytes())?;

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

                // Write previous_entry_id (64 bytes) at the end
                output.write_all(previous_entry.clone().as_bytes())?;
            }
        }
        output.flush()?;
        let (_, hash) = output.into_inner();
        Ok(hash.finalize().encode_hex())
    }

    pub(super) fn short_hash(&self) -> Result<String> {
        Ok(self.serialize(empty())?[..6].to_string())
    }

    pub(crate) fn deserialize<R: Read + Seek>(mut reader: R) -> Result<Self> {
        let mut reader: Box<dyn Read> = {
            let gzipper = GzDecoder::new(&mut reader);
            match gzipper.header() {
                None => {
                    reader.seek(std::io::SeekFrom::Start(0))?;
                    Box::new(reader)
                }
                Some(_) => Box::new(gzipper),
            }
        };
        let buffer: [u8; 8] = [0x00; 8];
        // Read discriminant
        read!(discriminant(u8) from reader using buffer);
        match discriminant {
            0x00 => {
                // Origin variant: need 8 bytes for year and 8 for timestamp
                read!(year(u64) from reader using buffer);
                read!(epoch_secs(i64) from reader using buffer);
                let timestamp = DateTime::from_timestamp(epoch_secs, 0).ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid timestamp")
                })?;
                Ok(Entry::Origin { year, timestamp })
            }
            0x01 => {
                // Read event_date (4 bytes, little endian)
                read!(date_days(i32) from reader using buffer);
                let event_date =
                    NaiveDate::from_num_days_from_ce_opt(date_days).ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid event date")
                    })?;

                // Read timestamp (8 bytes, little endian)
                read!(epoch_secs(i64) from reader using buffer);
                let timestamp = DateTime::from_timestamp(epoch_secs, 0).ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid timestamp")
                })?;

                read!(name_len(u32) as usize from reader using buffer);
                read!(desc_len(u32) as usize from reader using buffer);
                read!(name(name_len) as String from reader);
                read!(description(desc_len) as String from reader);
                read!(lines_count(u32) from reader using buffer);

                // Read lines
                let mut lines = Vec::new();
                for _ in 0..lines_count {
                    let line = EntryLine::deserialize(&mut reader)?;
                    lines.push(line);
                }
                read!(previous_entry(64) as String from reader);
                Ok(Entry::Entry {
                    timestamp,
                    event_date,
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

#[cfg(test)]
mod tests {
    use std::{env, fs::File, io::Cursor, ops::Deref};

    use super::*;
    use crate::Side;
    use chrono::{NaiveDate, TimeZone, Utc};
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    // Arbitrary impl for Side
    impl Arbitrary for EntryLine {
        fn arbitrary(g: &mut Gen) -> Self {
            let account = String::arbitrary(g);
            let amount = usize::arbitrary(g) % 10_000_000;
            let side = if bool::arbitrary(g) {
                Side::Debit
            } else {
                Side::Credit
            };
            let description = if bool::arbitrary(g) {
                Some(String::arbitrary(g))
            } else {
                None
            };
            super::EntryLine {
                account,
                amount,
                side,
                description,
            }
        }
    }
    impl Arbitrary for Side {
        fn arbitrary(g: &mut Gen) -> Self {
            if bool::arbitrary(g) {
                Side::Debit
            } else {
                Side::Credit
            }
        }
    }

    #[derive(Clone, Copy)]
    struct ArbDateTime(DateTime<Utc>);

    impl Deref for ArbDateTime {
        type Target = DateTime<Utc>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Arbitrary for ArbDateTime {
        fn arbitrary(g: &mut Gen) -> Self {
            let naive_date =
                NaiveDate::from_num_days_from_ce_opt((u32::arbitrary(g) % 3652425) as i32)
                    .unwrap_or_else(|| NaiveDate::from_ymd_opt(2020, 1, 1).unwrap())
                    .and_hms_opt(0, 0, 0)
                    .unwrap();
            ArbDateTime(Utc.from_utc_datetime(&naive_date))
        }
    }

    impl Arbitrary for Entry {
        fn arbitrary(g: &mut Gen) -> Self {
            if bool::arbitrary(g) {
                Entry::Origin {
                    timestamp: *ArbDateTime::arbitrary(g),
                    year: ArbDateTime::arbitrary(g).year() as u64,
                }
            } else {
                Entry::Entry {
                    timestamp: *ArbDateTime::arbitrary(g),
                    event_date: ArbDateTime::arbitrary(g).date_naive(),
                    name: String::arbitrary(g),
                    description: String::arbitrary(g),
                    lines: Vec::<EntryLine>::arbitrary(g),
                    previous_entry: (0..64)
                        .map(|_| char::arbitrary(g))
                        .collect::<String>()
                        .encode_hex::<String>()[0..64]
                        .to_string(),
                }
            }
        }
    }

    #[quickcheck]
    fn prop_entry_ser_de(entry: Entry) -> Result<bool> {
        let mut buf = Cursor::new(Vec::new());
        entry.serialize(&mut buf)?;
        buf.set_position(0);
        dbg!(&buf);
        let de = Entry::deserialize(&mut buf)?;
        Ok(entry == de)
    }

    #[test]
    fn simple_example() -> Result<()> {
        let mut g = Gen::new(2);
        let entry = Entry::Entry {
            timestamp: *ArbDateTime::arbitrary(&mut g),
            event_date: ArbDateTime::arbitrary(&mut g).date_naive(),
            name: "A entry".to_string(),
            description: "No description".to_string(),
            lines: vec![
                EntryLine::arbitrary(&mut g),
                EntryLine::arbitrary(&mut g),
                EntryLine::arbitrary(&mut g),
            ],
            previous_entry: "4f3e78b77d3a9bb2c1d305f4d536d4da2cd56adb2820af5b94ad3f9da0576b11"
                .to_string(),
        };
        let mut buf = Cursor::new(Vec::new());
        entry.serialize(&mut buf)?;
        buf.set_position(0);
        let de = Entry::deserialize(&mut buf)?;
        assert!(entry == de);
        Ok(())
    }

    #[quickcheck]
    fn hash_is_same(entry: Entry) -> Result<bool> {
        let mut buf = Cursor::new(Vec::new());
        let hash = entry.serialize(&mut buf)?;
        Ok(hash.starts_with(&entry.short_hash()?))
    }

    #[quickcheck]
    fn hash_is_same_multiple_times(a: Entry) -> Result<bool> {
        let hash = a.serialize(empty())?;
        let hash_2 = a.serialize(empty())?;
        Ok(hash == hash_2)
    }

    #[quickcheck]
    fn hash_is_same_multiple_times_different_buffer(a: Entry) -> Result<bool> {
        let buf = Cursor::new(Vec::new());
        let hash = a.serialize(buf)?;
        let hash_2 = a.serialize(empty())?;
        Ok(hash == hash_2)
    }

    #[quickcheck]
    fn hash_consistent_on_disk(entry: Entry) -> Result<bool> {
        let dir = env::temp_dir();
        let path = dir.join("temp_hash");
        dbg!(&path);
        let mut file = File::create(&path)?;
        let hash_1 = entry.serialize(&mut file)?;
        drop(file);
        let new_entry = Entry::from_file(&path)?;
        let hash_2 = new_entry.short_hash()?;
        Ok(hash_1.starts_with(&hash_2))
    }

    #[quickcheck]
    fn hash_is_different(a: Entry, b: Entry) -> Result<bool> {
        Ok(a.short_hash()? != b.short_hash()?)
    }
}
