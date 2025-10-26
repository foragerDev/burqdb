use crate::memory::{DBHeader, Frame};
use anyhow::{self, Result};
use std::fs::File as OsFile;
use std::io::Write;
use std::io::{Read, Seek};
use std::{cell::RefCell, rc::Rc};

const PageSize: i64 = 4096;

pub struct DbFile {
    // Later we will add configurations, for now let's do simple thing
    file: Rc<RefCell<OsFile>>,
    forced_sync: bool,
}

impl DbFile {
    pub fn new(file: Rc<RefCell<OsFile>>, forced_sync: bool) -> Self {
        DbFile { file, forced_sync }
    }

    pub fn read_page(&self, page_id: usize) -> Result<Frame> {
        self.seek(page_id)?;

        let mut buffer = vec![0u8; PageSize as usize];
        self.file.borrow_mut().read_exact(&mut buffer)?;

        Ok(Frame::from_bytes(buffer.into_boxed_slice()))
    }

    pub fn write_page(&self, page_id: usize, data: Box<[u8]>) -> Result<()> {
        self.seek(page_id)?;

        self.file.borrow_mut().write_all(&data)?;

        if self.forced_sync {
            self.file.borrow().sync_all()?;
        }

        Ok(())
    }

    pub fn seek(&self, page_id: usize) -> Result<()> {
        let offset = size_of::<DBHeader>() as u64 + (page_id as u64 * PageSize as u64);

        self.file
            .borrow_mut()
            .seek(std::io::SeekFrom::Start(offset))?;

        Ok(())
    }
}
