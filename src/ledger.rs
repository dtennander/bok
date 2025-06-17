use std::{io::Result, path::PathBuf, rc::Rc};

use crate::{Entry, EntryLine};

pub struct Ledger {
    head: Rc<Entry>,
    location: PathBuf,
}

impl Ledger {
    pub fn new(year: usize, location: PathBuf) -> Self {
        Self {
            head: Rc::new(Entry::Origin { year }),
            location,
        }
    }

    pub fn add_entry(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        lines: Vec<EntryLine>,
    ) -> Result<Rc<Entry>> {
        let new_entry = Entry::create_new(
            self.location.as_path(),
            name,
            description,
            lines,
            self.head.clone(),
        )?;
        self.head = Rc::new(new_entry);
        Ok(self.head.clone())
    }
}
