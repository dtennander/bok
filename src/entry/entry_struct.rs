use std::fs::read;
use std::io::Result;
use std::path::Path;

use chrono::{DateTime, NaiveDate, Utc};

use super::EntryLine;

/// Entry in the General Ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    Entry {
        timestamp: DateTime<Utc>,
        event_date: NaiveDate,
        name: String,
        description: String,
        lines: Vec<EntryLine>,
        previous_entry: String,
    },
    Origin {
        timestamp: DateTime<Utc>,
        year: u64,
    },
}

impl Entry {
    /// Constructor for the Entry::Entry variant
    pub fn new(
        date: NaiveDate,
        name: &str,
        description: &str,
        lines: Vec<EntryLine>,
        previous_entry: &str,
    ) -> Self {
        Entry::Entry {
            timestamp: chrono::Utc::now(),
            event_date: date,
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
