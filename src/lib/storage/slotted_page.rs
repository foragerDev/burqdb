use std::cell;

use anyhow::Result;
use bincode::{config::standard, serde::encode_to_vec};
use serde::{Deserialize, Serialize};
static PAGE_SIZE: usize = 4096;

pub struct PageHeader<K, V> {
    page_id: u64,
    right: Box<SlottedPage<K, V>>,
    page_size: usize,
}

impl<K, V> PageHeader<K, V>
where
    K: Serialize + for<'de> Deserialize<'de> + Ord,
    V: Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(page_id: u64, right: Box<SlottedPage<K, V>>) -> Self {
        Self {
            page_id,
            right,
            page_size: size_of::<PageHeader<K, V>>(),
        }
    }
}

pub struct Cell<K, V> {
    key: K,
    value: V,
    cached_size: Option<usize>,
}

impl<K, V> Cell<K, V>
where
    K: Serialize + for<'de> Deserialize<'de> + Ord,
    V: Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(key: K, value: V) -> Result<Self> {
        Ok(Self {
            key,
            value,
            cached_size: None,
        })
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        let key_bytes = encode_to_vec(&self.key, standard()).unwrap();
        let value_bytes = encode_to_vec(&self.value, standard()).unwrap();
        let mut buffer = Vec::new();

        buffer.extend_from_slice(&(key_bytes.len() as u32).to_le_bytes());
        buffer.extend_from_slice(&key_bytes);
        buffer.extend_from_slice(&value_bytes);

        Ok(buffer)
    }

    pub fn size(&mut self) -> usize {
        match self.cached_size {
            None => {
                self.cached_size = Some(self.serialize().unwrap().len() as usize);
                self.cached_size.unwrap()
            }
            Some(cached_size) => cached_size,
        }
    }
}

pub struct SlottedPage<K, V> {
    header: PageHeader<K, V>,
    offsets: Vec<usize>,
    cells: Vec<Cell<K, V>>,
    free_list: Vec<usize>,
}

impl<K, V> SlottedPage<K, V>
where
    K: Serialize + for<'de> Deserialize<'de> + Ord,
    V: Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(page_id: u64, right: Box<SlottedPage<K, V>>) -> Result<Self> {
        Ok(Self {
            header: PageHeader::new(page_id, right),
            offsets: Vec::new(),
            cells: Vec::new(),
            free_list: Vec::new(),
        })
    }

    pub fn can_insert(&self, cell: &mut Cell<K, V>) -> bool {
        let required_size = cell.size();
        if PAGE_SIZE - self.header.page_size >= required_size {
            true
        } else {
            self.free_list.iter().any(|&size| size >= required_size)
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<()> {

        
        let mut cell = Cell::new(key, value)?;
        if self.can_insert(&mut cell) == false {
            anyhow::bail!("Not enough space to insert the cell");
        }

        
        self.cells.push(cell);

        let offset = self.cells.len() - 1;
        self.offsets.push(offset);

        Ok(())
    }
}
