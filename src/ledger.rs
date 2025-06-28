use std::{
    collections::{HashMap, hash_map::Entry as HashEntry},
    fs::{self, create_dir_all, read, write},
    io::{Error, ErrorKind, Result},
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
    pub fn init(year: usize, location: PathBuf) -> Result<Self> {
        if location.is_dir() {
            return Err(Error::new(
                ErrorKind::DirectoryNotEmpty,
                "Directory isn't empty",
            ));
        }
        create_dir_all(&location)?;

        let head = Entry::Origin { year: year as u64 };
        let mut buffer = vec![];
        let hash = head.serialize(&mut buffer)?;
        let head_path = location.join("HEAD");
        write(&head_path, &hash)?;
        let object_path = location.join("objects");
        create_dir_all(&object_path)?;
        write(object_path.join(&hash), buffer)?;
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
            return Err(Error::new(
                std::io::ErrorKind::NotADirectory,
                "Directory doesn't exist",
            ));
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
        let new_head = Entry::new(name, description, lines, &self.head_hash);
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

    pub fn from_ref(&self, entry_ref: &str) -> Result<EntryHash> {
        if entry_ref == "HEAD" {
            return Ok(EntryHash(self.head_hash.clone()));
        }
        match &self.find_hash(entry_ref)?[..] {
            [] => Err(Error::new(ErrorKind::NotFound, "ref not found")),
            [entry_hash] => Ok(entry_hash.clone()),
            _ => Err(Error::new(ErrorKind::TooManyLinks, "To many refs match")),
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
                Ok(_) => None,          // Skip entries that don't match
                Err(e) => Some(Err(e)), // Propagate errors
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
            result += &entry.show_short();
            let next_ref = previous_entry.clone();
            next_hash = self.from_ref(&next_ref)?;
        }

        let last_entry = self.get_entry(&next_hash)?;
        result += &last_entry.show_short();
        Ok(result)
    }
}
