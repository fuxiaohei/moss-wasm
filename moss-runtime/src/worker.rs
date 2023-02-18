use crate::context::Context;
use anyhow::Result;
use moss_host_call::http_impl;
use wasmtime::component::{Component, InstancePre, Linker};
use wasmtime::{Config, Engine};

fn create_wasmtime_config() -> Config {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    config
}

pub struct Worker {
    _path: String,
    engine: Engine,
    // component: Component,
    instance_pre: InstancePre<Context>,
}

impl Worker {
    pub async fn new(path: &str) -> Result<Self> {
        // create component
        let config = create_wasmtime_config();
        let engine = Engine::new(&config)?;
        let component = Component::from_file(&engine, path)?;

        // create linker
        let mut linker: Linker<Context> = Linker::new(&engine);
        wasi_host::add_to_linker(&mut linker, Context::wasi)?;

        // create instance_pre
        let instance_pre = linker.instantiate_pre(&component)?;

        Ok(Self {
            _path: path.to_string(),
            engine,
            instance_pre,
        })
    }
    pub async fn execute(
        &mut self,
        req: http_impl::http_handler::Request<'_>,
    ) -> Result<http_impl::http_handler::Response> {
        // create store
        let mut store = wasmtime::Store::new(&self.engine, Context::new());

        // get exports and call handle_request
        let (exports, _instance) =
            http_impl::HttpHandler::instantiate_pre(&mut store, &self.instance_pre).await?;
        let resp = exports
            .http_handler()
            .call_handle_request(&mut store, req)
            .await?;
        Ok(resp)
    }
}
