use std::{
    collections::{HashMap, hash_map::Entry as HashEntry},
    fs::{self, create_dir_all, read, write},
    io::{Cursor, Error, ErrorKind},
    path::PathBuf,
};

use chrono::{Local, NaiveDate, Utc};

use crate::{
    BokError, Entry, EntryLine, Result,
    chart_of_accounts::{Account, ChartOfAccount},
};

pub struct Ledger {
    head: Entry,
    head_hash: String,
    object_path: PathBuf,
    head_path: PathBuf,

    hash_map: HashMap<String, Entry>,
}

#[derive(Debug, Clone)]
pub struct EntryHash(String);

impl AsRef<str> for EntryHash {
    #[inline]
    fn as_ref(&self) -> &str {
        <String as AsRef<str>>::as_ref(&self.0)
    }
}

pub enum ReferencedObject {
    Entry(EntryHash),
    Account(Account),
}

impl Ledger {
    pub fn init(year: usize, location: PathBuf, chart: ChartOfAccount) -> Result<Self> {
        if location.is_dir() {
            return Err(BokError::Initialization);
        }
        create_dir_all(&location)?;

        let head = Entry::Origin {
            timestamp: Utc::now(),
            year: year as u64,
            chart,
        };
        let mut buffer = Cursor::new(vec![]);
        let hash = head.serialize(&mut buffer)?;
        let head_path = location.join("HEAD");
        write(&head_path, &hash)?;
        let object_path = location.join("objects");
        create_dir_all(&object_path)?;
        write(object_path.join(&hash), buffer.into_inner())?;
        Ok(Self {
            head,
            head_hash: hash,
            head_path,
            object_path,
            hash_map: HashMap::new(),
        })
    }

    pub fn from_dir(location: PathBuf) -> Result<Self> {
        if !location.is_dir() {
            return Err(BokError::BadLedger);
        }
        let head_path = location.join("HEAD");
        let head_hash = String::from_utf8(read(&head_path)?)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Couldn't parse HEAD file..."))?;
        let object_path = location.join("objects");
        let head = Entry::from_file(&object_path.join(&head_hash))?;
        Ok(Self {
            head,
            head_hash,
            object_path,
            head_path,
            hash_map: HashMap::new(),
        })
    }

    pub fn add_entry(
        &mut self,
        name: &str,
        description: &str,
        lines: Vec<EntryLine>,
    ) -> Result<EntryHash> {
        self.add_entry_on_date(Local::now().date_naive(), name, description, lines)
    }

    pub fn add_entry_on_date(
        &mut self,
        date: NaiveDate,
        name: &str,
        description: &str,
        lines: Vec<EntryLine>,
    ) -> Result<EntryHash> {
        let new_head = Entry::new(date, name, description, lines, &self.head_hash);
        let mut buffer = Cursor::new(vec![]);
        let hash = new_head.serialize(&mut buffer)?;
        let path = self.object_path.join(&hash);
        create_dir_all(&self.object_path)?;
        write(path, buffer.into_inner())?;
        write(&self.head_path, &hash)?;
        self.head_hash = hash;
        self.head = new_head;
        Ok(EntryHash(self.head_hash.clone()))
    }

    pub fn from_ref(&mut self, entry_ref: &str) -> Result<ReferencedObject> {
        if entry_ref == "HEAD" {
            return Ok(ReferencedObject::Entry(EntryHash(self.head_hash.clone())));
        }
        if let Ok(account) = entry_ref.parse() {
            if let Some(account) = self
                .chart_of_accounts()?
                .iter()
                .find(|a| a.account_number == account)
            {
                return Ok(ReferencedObject::Account(account.clone()));
            }
        }
        match &self.find_hash(entry_ref)?[..] {
            [] => Err(BokError::RefNotFound),
            [entry_hash] => Ok(ReferencedObject::Entry(entry_hash.clone())),
            _ => Err(BokError::ToManyMatches),
        }
    }

    pub fn find_hash(&self, hash: &str) -> Result<Vec<EntryHash>> {
        fs::read_dir(&self.object_path)?
            .map(|r| {
                r.and_then(|d| {
                    d.file_name()
                        .into_string()
                        .map_err(|_| Error::new(std::io::ErrorKind::InvalidData, "BAD!"))
                })
            })
            .filter_map(|r| match r {
                Ok(s) if s.starts_with(hash) => Some(Ok(EntryHash(s))),
                Ok(_) => None,                 // Skip entries that don't match
                Err(e) => Some(Err(e.into())), // Propagate errors
            })
            .collect()
    }

    pub fn get_entry(&mut self, hash: &EntryHash) -> Result<&Entry> {
        match self.hash_map.entry(hash.0.clone()) {
            HashEntry::Vacant(ve) => {
                let entry_file = self.object_path.join(hash.0.clone());
                let entry_ref = ve.insert(Entry::from_file(&entry_file)?);
                Ok(entry_ref)
            }
            HashEntry::Occupied(o) => Ok(o.into_mut()),
        }
    }

    pub fn show_log(&mut self, hash: EntryHash) -> Result<String> {
        let mut next_hash = hash;
        let mut result = String::new();

        while let entry @ Entry::Entry { previous_entry, .. } = self.get_entry(&next_hash)? {
            result += &entry.show_short()?;
            let next_ref = previous_entry.clone();
            next_hash = self.find_hash(&next_ref)?[0].clone();
        }

        let last_entry = self.get_entry(&next_hash)?;
        result += &last_entry.show_short()?;
        Ok(result)
    }

    /// Returns the chart of accounts from the origin entry
    pub fn chart_of_accounts(&mut self) -> Result<&ChartOfAccount> {
        let first_entry = self.head_hash.clone();
        let origin_hash = {
            let mut hash = self.find_hash(&first_entry)?[0].clone();
            while let Entry::Entry { previous_entry, .. } = self.get_entry(&hash)? {
                let entry_ref = previous_entry.clone();
                hash = self.find_hash(&entry_ref)?[0].clone();
            }
            hash
        };
        if let Entry::Origin { chart, .. } = self.get_entry(&origin_hash)? {
            return Ok(chart);
        }
        Err(BokError::BadLedger)
    }

    /// Lists all entries with their short hash and description.
    /// No ordering is used here
    pub fn list_entries(&mut self) -> Result<Vec<(String, String)>> {
        let all_hashes = self.find_hash("")?;
        let mut result = Vec::new();
        for hash in all_hashes {
            let entry = self.get_entry(&hash)?;
            let short_hash = &hash.0[..6.min(hash.0.len())];
            let description = match entry {
                Entry::Entry { description, .. } => description.clone(),
                Entry::Origin { year, .. } => format!("Origin ({})", year),
            };
            result.push((short_hash.to_string(), description));
        }
        Ok(result)
    }
}
