//! #Cell
//!
//! Structure for holding Key Value pair for hold row against the key
//! the idea is simple key will be used to identify the row (value) serialized

use anyhow::Result;
use bincode::{config::standard, serde::encode_to_vec};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Cell<K, V> {
    key: K,
    value: V,

    #[serde(skip)]
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

        // buffer.extend_from_slice(&(key_bytes.len() as u32).to_le_bytes());
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
