use std::{
    fs::{create_dir_all, write},
    io::Result,
    path::PathBuf,
    rc::Rc,
};

use crate::{Entry, EntryLine};

pub struct Ledger {
    head: Rc<Entry>,
    object_path: PathBuf,
    head_path: PathBuf,
}

impl Ledger {
    pub fn new(year: usize, location: PathBuf) -> Self {
        Self {
            head: Rc::new(Entry::Origin { year }),
            object_path: location.join("objects"),
            head_path: location.join("HEAD"),
        }
    }

    pub fn add_entry(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        lines: Vec<EntryLine>,
    ) -> Result<Entry> {
        let new_head = Entry::create_new(name, description, lines, self.head.clone())?;
        let mut buffer: Vec<u8> = vec![];
        let hash = new_head.serialize(&mut buffer)?;
        let path = self.object_path.join(hash);
        create_dir_all(&self.object_path)?;
        write(path, buffer)?;
        self.switch_head(&new_head)?;
        Ok(new_head)
    }

    fn switch_head(&mut self, new_head: &Entry) -> Result<()> {
        self.head = Rc::new(new_head.clone());
        write(&self.head_path, new_head.get_hash_hex())
    }

    pub fn persist() -> Result<()> {
        Ok(())
    }
}
