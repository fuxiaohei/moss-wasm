use moss_host_call::fetch_impl::FetchImpl;
use moss_host_call::kv_impl::{KvStorageImpl, Provider};
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_host::WasiCtx;

pub struct Context {
    wasi: WasiCtx,
    fetch_impl: FetchImpl,
    kv_storage: KvStorageImpl,
}

impl Default for Context {
    fn default() -> Self {
        Self::new(Some(super::KV_STORAGE.clone()))
    }
}

impl Context {
    pub fn new(kv_provider: Option<Provider>) -> Self {
        let provider = kv_provider.unwrap_or_else(|| super::KV_STORAGE.clone());
        Context {
            wasi: WasiCtxBuilder::new().inherit_stdio().build(),
            fetch_impl: FetchImpl::new(1),
            kv_storage: KvStorageImpl::new(provider),
        }
    }
    /// get wasi
    pub fn wasi(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
    /// get fetch impl
    pub fn fetch_impl(&mut self) -> &mut FetchImpl {
        &mut self.fetch_impl
    }
    /// get storage impl
    pub fn kv_storage(&mut self) -> &mut KvStorageImpl {
        &mut self.kv_storage
    }
}
