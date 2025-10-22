// use std::collections::HashMap;

// use anyhow::Result;
// use serde::{self, Deserialize, Serialize};

// use crate::storage::cell::Cell;

// static PAGE_SIZE: u16 = 4095;

// //Currently let's keep it simple with only one page type later we can implement index pages
// #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
// pub enum PageType {
//     Leaf,
//     Internal,
// }

// #[derive(Serialize, Deserialize)]
// pub enum Position {
//     Free(u16),
//     Occupied(u16),
// }

// #[derive(Serialize, Deserialize)]
// pub struct PageHeader<K, V> {
//     page_type: PageType,
//     page_id: u64,
//     right: Option<Box<SlottedPage<K, V>>>,
//     page_size: u16,
//     offset: u16,
//     first_freeblock: u16,
// }


// impl<K, V> PageHeader<K, V>
// where
//     K: Serialize + for<'de> Deserialize<'de> + Ord,
//     V: Serialize + for<'de> Deserialize<'de>,
// {
//     pub fn new(page_id: u64, right: Option<Box<SlottedPage<K, V>>>, page_type: PageType) -> Self {
//         Self {
//             page_type,
//             page_id,
//             right,
//             page_size: 0,
//             offset: PAGE_SIZE as u16,
//             first_freeblock: 0,
//         }
//     }
// }

// #[derive(Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
// pub struct FreeBlock {
//     offset: u16,
//     size: u16,
// }

// impl FreeBlock {
//     pub fn split(&mut self, size: u16) {}
// }

// #[derive(Serialize, Deserialize)]
// pub struct SlottedPage<K, V> {
//     header: PageHeader<K, V>,
//     offsets: Vec<usize>,
//     cells: HashMap<u16, Cell<K, V>>,
//     freeblocks: Vec<FreeBlock>,
// }

// impl<K, V> SlottedPage<K, V>
// where
//     K: Serialize + for<'de> Deserialize<'de> + Ord,
//     V: Serialize + for<'de> Deserialize<'de>,
// {
//     pub fn new(
//         page_id: u64,
//         right: Option<Box<SlottedPage<K, V>>>,
//         page_type: PageType,
//     ) -> Result<Self> {
//         Ok(Self {
//             header: PageHeader::new(page_id, right, page_type),
//             offsets: Vec::new(),
//             cells: HashMap::new(),
//             freeblocks: Vec::new(),
//         })
//     }

//     pub fn page_id(&self) -> u64 {
//         self.header.page_id
//     }

//     pub fn can_insert(&mut self, cell: &mut Cell<K, V>) -> Result<Position> {
//         let required_bytes = cell.size() as u16;
//         let header_side =
//             size_of::<PageHeader<K, V>>() as u16 + (self.offsets.len() as u16 + 1) * 2;

//         if self.header.offset - header_side >= required_bytes + 2 {
//             Ok(Position::Free(required_bytes as u16))
//         } else {
//             let mut freeblock;
//             {
//                 freeblock = self
//                     .freeblocks
//                     .iter()
//                     .enumerate()
//                     .find(|(_, block)| required_bytes <= block.size)
//                     .and_then(f);
//             }
//             if let Some((index, block)) = freeblock {
//                 let remaining_bytes = block.size - required_bytes;

//                 if remaining_bytes.ge(&4) {
//                     self.freeblocks[index].size = remaining_bytes;
//                 }

//                 Ok(Position::Occupied(required_bytes - block.offset))
//             } else {
//                 Err(anyhow::anyhow!("Not enough space to insert the cell"))
//             }
//         }
//     }

//     pub fn remove(&mut self, key: &K) -> Result<()> {
//         match self.find_key_index(key) {
//             Some(mut cell) => {
//                 let offset = self.offsets.remove(cell as usize);
//                 let mut cell = self.cells.remove(&(offset as u16)).unwrap();
//                 let size = cell.size() as usize;
//                 self.free_list.push(size);
//                 self.header.page_size -= size as u16;
//                 self.header.page_size -= 1;
//                 Ok(())
//             }
//             None => anyhow::bail!("No such key found"),
//         }
//     }

//     pub fn insert(&mut self, key: K, value: V) -> Result<()> {
//         let mut cell = Cell::new(key, value)?;
//         match self.can_insert(&mut cell) {
//             Ok(pos) => match pos {
//                 Position::Free(size) => {
//                     self.header.offset -= size + 1;
//                     let offset = self.header.offset + 1;
//                     self.header.page_size += size;
//                     self.offsets.push(offset as usize);
//                     self.cells.insert(offset, cell);
//                 }
//                 Position::Occupied(at) => {
//                     let index = self
//                         .free_list
//                         .iter()
//                         .position(|&s| s == at as usize)
//                         .unwrap();
//                     self.free_list.remove(index);
//                     self.header.page_size += at;
//                     self.offsets.push(at as usize);
//                     self.cells.insert(at, cell);
//                 }
//             },
//             Err(_) => anyhow::bail!("Not enough space to insert the cell"),
//         }

//         self.header.page_size += 1;
//         self.offsets.sort_by(|a, b| {
//             let cell_a = self.cells.get(&(*a as u16)).unwrap();
//             let cell_b = self.cells.get(&(*b as u16)).unwrap();
//             cell_a.key.cmp(&cell_b.key)
//         });
//         Ok(())
//     }

//     // kind of upper bound, return less than key suppose if cells are 1,3,5,6 if 2 is searched it should return 1
//     pub fn find_key(&self, key: &K) -> Option<&Cell<K, V>> {
//         let mut left = 0u16;
//         let mut right = self.offsets.len() as u16 - 1;
//         let mut result: Option<&Cell<K, V>> = None;

//         while left <= right {
//             let mid = (left + right) / 2;
//             let cell = self
//                 .cells
//                 .get(&(self.offsets[mid as usize] as u16))
//                 .unwrap();
//             if &cell.key == key {
//                 return Some(cell);
//             } else if &cell.key < key {
//                 result = Some(cell);
//                 left = mid + 1;
//             } else {
//                 right = mid - 1;
//             }
//         }

//         match result {
//             Some(cell) => Some(cell),
//             None => None,
//         }
//     }

//     pub fn find_key_index(&self, key: &K) -> Option<u16> {
//         let mut left = 0;
//         let mut right = self.offsets.len() as i32 - 1;

//         while left <= right {
//             let mid = (left + right) / 2;
//             let cell = self
//                 .cells
//                 .get(&(self.offsets[mid as usize] as u16))
//                 .unwrap();
//             if &cell.key == key {
//                 return Some(mid as u16);
//             } else if &cell.key < key {
//                 left = mid + 1;
//             } else {
//                 right = mid - 1;
//             }
//         }
//         None
//     }
// }

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn test_cell_serialization() {
//         let cell = Cell::new("key1".to_string(), "value1".to_string()).unwrap();
//         let serialized = cell._serialize().unwrap();
//         assert!(!serialized.is_empty());
//     }

//     #[test]
//     fn test_cell_size() {
//         let mut cell: Cell<String, String> =
//             Cell::new("key12".to_string(), "value1".to_string()).unwrap();
//         assert_eq!(cell.size(), 13);
//     }

//     #[test]
//     fn test_cell_size_int() {
//         let mut cell: Cell<i32, i32> = Cell::new(1, 1).unwrap();
//         assert_eq!(cell.size(), 2);
//     }
//     #[test]
//     fn test_add_cell() {
//         let mut page = SlottedPage::new(0, None, PageType::Internal).unwrap();
//         let result = page.insert("key1".to_string(), "value1".to_string());
//         assert!(result.is_ok());
//     }

//     #[test]
//     fn test_find_cell() {
//         let mut page = SlottedPage::new(0, None, PageType::Internal).unwrap();
//         page.insert("key1".to_string(), "value1".to_string())
//             .unwrap();
//         let cell = page.find_key(&"key1".to_string());
//         assert!(cell.is_some());
//     }

//     #[test]
//     fn test_multiple_cells() {
//         let mut page = SlottedPage::new(0, None, PageType::Internal).unwrap();
//         page.insert("key1".to_string(), "value1".to_string())
//             .unwrap();
//         page.insert("key2".to_string(), "value2".to_string())
//             .unwrap();
//         page.insert("key3".to_string(), "value3".to_string())
//             .unwrap();

//         let cell1 = page.find_key(&"key1".to_string());
//         let cell2 = page.find_key(&"key2".to_string());
//         let cell3 = page.find_key(&"key3".to_string());

//         assert!(cell1.is_some());
//         assert!(cell2.is_some());
//         assert!(cell3.is_some());
//     }

//     #[test]
//     fn test_offset_order() {
//         println!("test offset order");
//         let mut page = SlottedPage::new(0, None, PageType::Internal).unwrap();
//         page.insert("b".to_string(), "value1".to_string()).unwrap();
//         page.insert("c".to_string(), "value2".to_string()).unwrap();
//         page.insert("a".to_string(), "value3".to_string()).unwrap();

//         let s = format!("{:?}", page.offsets.clone());
//         println!("{}", s);
//         let offsets = page.offsets.clone();
//         assert_eq!(offsets, vec![4066, 4086, 4076]);
//     }

//     #[test]
//     fn test_full_page() {
//         let mut page = SlottedPage::new(0, None, PageType::Internal).unwrap();
//         let mut index = 0;
//         let mut error = anyhow::anyhow!("No error");
//         while match page.insert(format!("key{}", index), format!("value{}", index)) {
//             Ok(_) => {
//                 index += 1;
//                 true
//             }
//             Err(err) => {
//                 error = err;
//                 false
//             }
//         } {}
//         assert!(
//             error
//                 .to_string()
//                 .contains("Not enough space to insert the cell")
//         );
//     }

//     #[test]
//     fn test_insert_in_freelist() {
//         let mut page = SlottedPage::new(0, None, PageType::Internal).unwrap();
//         let mut index = 0;
//         let mut error = anyhow::anyhow!("No error");
//         while match page.insert(format!("key{}", index), format!("value{}", index)) {
//             Ok(_) => {
//                 index += 1;
//                 true
//             }
//             Err(err) => {
//                 error = err;
//                 false
//             }
//         } {}

//         let delete_cells = vec!["key11", "key10", "key4", "key3", "key2"];
//         for key in delete_cells {
//             assert!(page.remove(&key.to_string()).is_ok());
//         }

//         assert_eq!(page.free_list.len(), 5);
//         assert_eq!(page.free_list.iter().sum::<usize>(), 27);
//         assert_eq!(page.header.page_size, 3870);
//         assert_eq!(page.offsets.len(), index - 5);
//     }
// }
