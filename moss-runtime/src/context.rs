use moss_host_call::fetch_impl::FetchImpl;
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_host::WasiCtx;
pub struct Context {
    wasi: WasiCtx,
    fetch_impl: FetchImpl,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Context {
            wasi: WasiCtxBuilder::new().inherit_stdio().build(),
            fetch_impl: FetchImpl::new(1),
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
}
