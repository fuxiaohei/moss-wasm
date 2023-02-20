wasmtime::component::bindgen!({
    world:"kv-storage",
    path: "../../wit/kv-storage.wit",
    async: true,
});

use kv_storage::{Key, KvError, Pair, Value};
use moss_storage::kv;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type Provider = Arc<Mutex<dyn kv::KvStorage>>;

pub struct KvStorageImpl {
    storage: Provider,
}

impl KvStorageImpl {
    pub fn new(storage: Provider) -> Self {
        KvStorageImpl { storage }
    }
    pub fn is_expired(&self, t: u64) -> bool {
        if t == 0 {
            return false;
        }
        get_now_unixstamp() > t
    }
}

fn get_now_unixstamp() -> u64 {
    let now = std::time::SystemTime::now();
    now.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
}

/// convert kv::KvError to kv_storage::KvError
impl From<kv::KvError> for kv_storage::KvError {
    fn from(e: kv::KvError) -> Self {
        match e {
            kv::KvError::KeyNotFound => kv_storage::KvError::KeyNotFound,
            kv::KvError::InternalError => kv_storage::KvError::InternalError,
            kv::KvError::ValueTooLarge => kv_storage::KvError::ValueTooLarge,
            kv::KvError::InvalidKey => kv_storage::KvError::InvalidKey,
        }
    }
}

#[async_trait::async_trait]
impl kv_storage::KvStorage for KvStorageImpl {
    async fn get(&mut self, k: Key) -> anyhow::Result<Result<Value, KvError>> {
        let mut store = self.storage.lock().await;
        let value = match store.get(k).await {
            Ok(v) => v,
            Err(e) => return Ok(Err(e.into())),
        };
        if self.is_expired(value.1) {
            return Ok(Err(KvError::KeyNotFound));
        }
        Ok(Ok(value.0))
    }
    async fn set(&mut self, k: Key, v: Value, expire: u64) -> anyhow::Result<Result<(), KvError>> {
        let expire = if expire > 0 {
            get_now_unixstamp() + expire
        } else {
            0
        };
        let mut store = self.storage.lock().await;
        match store.set(k, (v, expire)).await {
            Ok(_) => return Ok(Ok(())),
            Err(e) => return Ok(Err(e.into())),
        }
    }
    async fn delete(&mut self, k: Key) -> anyhow::Result<Result<(), KvError>> {
        let mut store = self.storage.lock().await;
        match store.delete(k).await {
            Ok(_) => return Ok(Ok(())),
            Err(e) => return Ok(Err(e.into())),
        }
    }
    async fn get_all(&mut self) -> anyhow::Result<Result<Vec<Pair>, KvError>> {
        let mut store = self.storage.lock().await;
        let values = match store.get_all().await {
            Ok(v) => v,
            Err(e) => return Ok(Err(e.into())),
        };
        let mut pairs = Vec::new();
        for (k, (v, expire)) in values {
            if self.is_expired(expire) {
                continue;
            }
            pairs.push((k, v));
        }
        Ok(Ok(pairs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kv_impl::kv_storage::KvStorage;

    #[tokio::test]
    async fn run_kv_storage_impl() {
        let storage = kv::MemoryKvStorage::new();
        let storage = Arc::new(Mutex::new(storage));
        let mut kv_storage = KvStorageImpl::new(storage);
        kv_storage
            .set("abc".to_string(), "abcd".as_bytes().to_vec(), 100)
            .await
            .unwrap()
            .unwrap();
        let value = kv_storage.get("abc".to_string()).await.unwrap().unwrap();
        assert_eq!(value, "abcd".as_bytes().to_vec());

        // get not exist key
        let value = kv_storage.get("not_exist".to_string()).await.unwrap();
        assert!(value.is_err());

        let values = kv_storage.get_all().await.unwrap().unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].0, "abc");
        assert_eq!(values[0].1, "abcd".as_bytes().to_vec());

        kv_storage.delete("abc".to_string()).await.unwrap().unwrap();
        let values = kv_storage.get_all().await.unwrap().unwrap();
        assert_eq!(values.len(), 0);
    }

}
