use std::{fs::write, io::Result, path::PathBuf, rc::Rc};

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
    ) -> Result<Entry> {
        let new_head = Entry::create_new(
            self.location.as_path(),
            name,
            description,
            lines,
            self.head.clone(),
        )?;
        self.switch_head(&new_head)?;
        Ok(new_head)
    }

    fn switch_head(&mut self, new_head: &Entry) -> Result<()> {
        self.head = Rc::new(new_head.clone());
        let head_path = self.location.join("HEAD");
        write(head_path, new_head.get_hash_hex())
    }
}
