use crate::{Key, KvError, KvStorage, Pair, Value};
use std::collections::HashMap;
use tracing::debug;

/// MEMORY_KV_VALUE_MAX_SIZE is the maximum size of the value in the memory key-value store.
const MEMORY_KV_VALUE_MAX_SIZE: usize = 1024 * 1024;
/// MEMORY_KV_KEY_MAX_SIZE is the maximum size of the key in the memory key-value store.
const MEMORY_KV_KEY_MAX_SIZE: usize = 1024;

#[derive(Debug)]
pub struct MemoryKvStorage {
    data: HashMap<Key, Value>,
}

impl MemoryKvStorage {
    pub fn new() -> Self {
        debug!("[kv] init MemoryKvStorage");
        MemoryKvStorage {
            data: HashMap::new(),
        }
    }
}

impl Default for MemoryKvStorage {
    fn default() -> Self {
        MemoryKvStorage::new()
    }
}

#[async_trait::async_trait]
impl KvStorage for MemoryKvStorage {
    async fn get(&mut self, k: Key) -> Result<Value, KvError> {
        self.data.get(&k).cloned().ok_or(KvError::KeyNotFound)
    }
    async fn set(&mut self, k: Key, v: Value) -> Result<(), KvError> {
        if k.len() > MEMORY_KV_KEY_MAX_SIZE {
            return Err(KvError::InvalidKey);
        }
        if v.0.len() > MEMORY_KV_VALUE_MAX_SIZE {
            return Err(KvError::ValueTooLarge);
        }
        self.data.insert(k, v);
        Ok(())
    }
    async fn delete(&mut self, k: Key) -> Result<(), KvError> {
        self.data.remove(&k);
        Ok(())
    }
    async fn get_all(&mut self) -> Result<Vec<Pair>, KvError> {
        Ok(self
            .data
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_memory_kv() {
        let mut storage = MemoryKvStorage::new();
        storage
            .set("abc".to_string(), ("abc".as_bytes().to_vec(), 111))
            .await
            .unwrap();
        let value = storage.get("abc".to_string()).await.unwrap();
        assert_eq!(value.0, "abc".as_bytes().to_vec());
        assert_eq!(value.1, 111);

        let values = storage.get_all().await.unwrap();
        assert_eq!(values.len(), 1);

        storage.delete("abc".to_string()).await.unwrap();
        let values = storage.get_all().await.unwrap();
        assert_eq!(values.len(), 0);
    }
}
