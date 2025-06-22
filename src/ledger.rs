use std::{
    collections::{HashMap, hash_map::Entry as HashEntry},
    fs::{self, DirEntry, create_dir_all, read, write},
    io::{Error, Result},
    path::PathBuf,
};

use crate::{Entry, EntryLine};

pub struct Ledger {
    head: Entry,
    head_hash: String,
    object_path: PathBuf,
    head_path: PathBuf,

    hash_map: HashMap<String, Entry>,
}

#[derive(Clone)]
pub struct EntryHash(String);

impl AsRef<str> for EntryHash {
    #[inline]
    fn as_ref(&self) -> &str {
        <String as AsRef<str>>::as_ref(&self.0)
    }
}

impl Ledger {
    pub fn new(year: usize, location: PathBuf) -> Self {
        let head = Entry::Origin { year };
        let head_hash = head.get_hash_hex();
        Self {
            head,
            head_hash,
            object_path: location.join("objects"),
            head_path: location.join("HEAD"),
            hash_map: HashMap::new(),
        }
    }

    pub fn add_entry(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        lines: Vec<EntryLine>,
    ) -> Result<EntryHash> {
        let new_head = Entry::create_new(name, description, lines, self.head_hash.clone())?;
        let mut buffer: Vec<u8> = vec![];
        let hash = new_head.serialize(&mut buffer)?;
        let path = self.object_path.join(&hash);
        create_dir_all(&self.object_path)?;
        write(path, buffer)?;
        write(&self.head_path, &hash)?;
        self.head_hash = hash;
        self.head = new_head;
        Ok(EntryHash(self.head_hash.clone()))
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
                Ok(_) => None,          // Skip entries that don't match
                Err(e) => Some(Err(e)), // Propagate errors
            })
            .collect()
    }

    pub fn get_entry(&mut self, hash: &EntryHash) -> Result<&Entry> {
        match self.hash_map.entry(hash.0.clone()) {
            HashEntry::Vacant(ve) => {
                let bytes = read(self.object_path.join(hash.0.clone()))?;
                let val = Entry::from_bytes(&bytes)?;
                let entry_ref = ve.insert(val);
                Ok(entry_ref)
            }
            HashEntry::Occupied(o) => Ok(o.into_mut()),
        }
    }
}
