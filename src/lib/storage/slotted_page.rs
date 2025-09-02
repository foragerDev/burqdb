use std::{cell, collections::HashMap, hash::Hash};

use anyhow::Result;
use bincode::{config::standard, serde::encode_to_vec};
use serde::{Deserialize, Serialize};
static PAGE_SIZE: usize = 4096;

#[derive(Serialize, Deserialize)]

pub struct PageHeader<K, V> {
    page_id: u64,
    right: Box<SlottedPage<K, V>>,
    page_size: usize,
    offset: u16,
}

#[derive(Serialize, Deserialize)]

pub enum Position {
    Free(u16),
    Occupied(u16),
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
            offset: PAGE_SIZE as u16 - 1,
        }
    }
}

#[derive(Serialize, Deserialize)]
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

    pub fn _serialize(&self) -> Result<Vec<u8>> {
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
                self.cached_size = Some(self._serialize().unwrap().len() as usize);
                self.cached_size.unwrap()
            }
            Some(cached_size) => cached_size,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SlottedPage<K, V> {
    header: PageHeader<K, V>,
    offsets: Vec<usize>,
    cells: HashMap<u16, Cell<K, V>>,
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
            cells: HashMap::new(),
            free_list: Vec::new(),
        })
    }

    pub fn can_insert(&self, cell: &mut Cell<K, V>) -> Result<Position> {
        let required_size = cell.size();
        if PAGE_SIZE - self.header.page_size >= required_size {
            Ok(Position::Free(required_size as u16))
        } else {
            let r = self
                .free_list
                .iter()
                .find(move |&&size| size >= required_size);
            match r {
                Some(&size) => Ok(Position::Occupied(size as u16)),
                None => Err(anyhow::anyhow!("Not enough space to insert the cell")),
            }
        }
    }


    

    pub fn insert(&mut self, key: K, value: V) -> Result<()> {
        let mut cell = Cell::new(key, value)?;
        match self.can_insert(&mut cell) {
            Ok(pos) => match pos {
                Position::Free(size) => {
                    self.header.offset -= size;
                    self.header.page_size += size as usize;
                    self.offsets.push(size as usize);
                    self.cells.insert(size, cell);
                }
                Position::Occupied(size) => {
                    let index = self
                        .free_list
                        .iter()
                        .position(|&s| s == size as usize)
                        .unwrap();
                    self.free_list.remove(index);
                    self.header.page_size += size as usize;
                    self.offsets.push(size as usize);
                    self.cells.insert(size, cell);
                }
            },
            Err(_) => anyhow::bail!("Not enough space to insert the cell"),
        }

        self.offsets.sort_by(|a, b| {
            let cell_a = self.cells.get(&(*a as u16)).unwrap();
            let cell_b = self.cells.get(&(*b as u16)).unwrap();
            cell_a.key.cmp(&cell_b.key)
        });
        Ok(())
    }

    // kind of upper bound, return less than key suppose if cells are 1,3,5,6 if 2 is searched it should return 1
    pub fn find_pos(&self, key: &K) -> Result<&Cell<K, V>> {
        let mut left = 0u16;
        let mut right = self.offsets.len() as u16 - 1;
        let mut result: Option<&Cell<K, V>> = None;

        while left <= right {
            let mid = (left + right) / 2;
            let cell = self.cells.get(&(self.offsets[mid as usize] as u16)).unwrap();
            if &cell.key == key {
                return Ok(cell);
            } else if &cell.key < key {
                result = Some(cell);
                left = mid + 1;
            } else {
                right = mid - 1;
            }
        }

        match result {
            Some(cell) => Ok(cell),
            None => Err(anyhow::anyhow!("No such key found")),
        }

    }

    pub fn find_cell(&self, key: &K) -> Option<&Cell<K, V>> {
        let mut left = 0u16;
        let mut right = self.offsets.len() as u16 - 1;

        while left <= right {
            let mid = (left + right) / 2;
            let cell = self.cells.get(&(self.offsets[mid as usize] as u16)).unwrap();
            if &cell.key == key {
                return Some(cell);
            } else if &cell.key < key {
                left = mid + 1;
            } else {
                right = mid - 1;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_serialization() {
        let cell = Cell::new("key1".to_string(), "value1".to_string()).unwrap();
        let serialized = cell._serialize().unwrap();
        assert!(!serialized.is_empty());
    }

    // #[test]
    // fn test_cell_deserialization() {
    //     let cell = Cell::new("key1".to_string(), "value1".to_string()).unwrap();
    //     let serialized = cell._serialize().unwrap();
    //     // let deserialized: Cell<String, String> = bincode::serde::deserialize(&serialized).unwrap();
    //     assert_eq!(cell, deserialized);
    // }
}
