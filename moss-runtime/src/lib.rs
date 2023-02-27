pub mod compiler;
pub mod context;
pub mod pool;
pub mod worker;

/// create global kv provider
use moss_host_call::kv_impl::Provider;
use moss_kv_service::MemoryKvStorage;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;

// KV_STORAGE is a global kv
static KV_STORAGE: Lazy<Provider> = Lazy::new(|| Arc::new(Mutex::new(MemoryKvStorage::new())));
