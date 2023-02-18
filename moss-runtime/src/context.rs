use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_host::WasiCtx;
pub struct Context {
    wasi: WasiCtx,
}

impl Context {
    pub fn new() -> Self {
        Context {
            wasi: WasiCtxBuilder::new().inherit_stdio().build(),
        }
    }
    /// get wasi
    pub fn wasi(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}
