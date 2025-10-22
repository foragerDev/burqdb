use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::path::Path;
use std::{cell::RefCell, rc::Rc};

use anyhow::{Context, Result};

const DB_EXTENTION: &'static str = "bd";

#[derive(Default)]
struct FileManager {
    files: HashMap<String, Rc<RefCell<File>>>,
}

impl FileManager {
    /// Creates a file at given `path` and with given `name`
    pub fn create(path: &Path, name: &str) -> Result<()> {
        let file_path = path.join(format!("{}.{}", name, DB_EXTENTION));
        let _ = OpenOptions::new()
            .create_new(true)
            .open(file_path)
            .with_context(|| format!("Failed to create file at {}", &path.display()));

        Ok(())
    }

    pub fn delete(path: &Path, name: &str) -> Result<()> {
        let file_path = path.join(format!("{}.{}", name, DB_EXTENTION));
        Ok(fs::remove_file(file_path)?)
    }

    // Open the given file, do not pass the extention the only thing important is file path
    // Probably there will be a default path were database will be created?
    pub fn open(&mut self, path: &Path, name: &str) -> Result<Rc<RefCell<File>>> {
        let key = format!("{}:{}", path.display(), name);

        if let Some(file_rc) = self.files.get(&key) {
            return Ok(Rc::clone(file_rc));
        }

        let file_path = path.join(format!("{}.{}", name, DB_EXTENTION));
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(file_path)
            .with_context(|| format!("Failed to open file at {}", &path.display()))?;

        let file_rc = Rc::new(RefCell::new(file));
        self.files.insert(key, Rc::clone(&file_rc));
        Ok(file_rc)
    }
}

#[cfg(test)]
mod tests {
    use std::{path::Path};

    use crate::io::file_manager::FileManager;

    #[test]
    fn test_file_create_new_file() {
        let path = Path::new("/tmp");
        FileManager::create(path, "shop");

        let file_path = path.join("shop");

        assert!(file_path.exists());        
    }
}
