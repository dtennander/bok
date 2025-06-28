use std::fs::read;
use std::io::Result;
use std::path::Path;

use super::EntryLine;

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
        year: u64,
    },
}

impl Entry {
    /// Constructor for the Entry::Entry variant
    pub fn new(name: &str, description: &str, lines: Vec<EntryLine>, previous_entry: &str) -> Self {
        Entry::Entry {
            name: name.to_string(),
            description: description.to_string(),
            lines,
            previous_entry: previous_entry.to_string(),
        }
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let bytes = read(path)?;
        Self::from_bytes(&bytes)
    }
}
