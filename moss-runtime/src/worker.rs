use crate::context::Context;
use anyhow::Result;
use moss_host_call::fetch_impl;
use moss_host_call::http_impl;
use moss_host_call::kv_impl;
use wasmtime::component::{Component, InstancePre, Linker};
use wasmtime::{Config, Engine, Store};

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
        fetch_impl::http_fetch::add_to_linker(&mut linker, Context::fetch_impl)?;
        kv_impl::kv_storage::add_to_linker(&mut linker, Context::kv_storage)?;

        // create instance_pre
        let instance_pre = linker.instantiate_pre(&component)?;

        let worker = Self {
            _path: path.to_string(),
            engine,
            instance_pre,
        };

        Ok(worker)
    }

    pub async fn handle_request(
        &mut self,
        req: http_impl::http_handler::Request<'_>,
    ) -> Result<http_impl::http_handler::Response> {
        // create store
        let mut store = Store::new(&self.engine, Context::new(None));

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

#[cfg(test)]
mod tests {
    use super::Worker;
    use moss_host_call::http_impl::http_handler::Request;

    #[tokio::test]
    async fn run_wasm() {
        let wasm_file = "../tests/data/rust_basic.component.wasm";
        let mut worker = Worker::new(wasm_file).await.unwrap();

        for _ in 1..10 {
            let headers: Vec<(&str, &str)> = vec![];
            let req = Request {
                method: "GET",
                uri: "/abc",
                headers: &headers,
                body: Some("xxxyyy".as_bytes()),
            };

            let resp = worker.handle_request(req).await.unwrap();
            assert_eq!(resp.status, 200);
            assert_eq!(resp.body, Some("Hello, World".as_bytes().to_vec()));

            let headers = resp.headers;
            for (key, value) in headers {
                if key == "X-Request-Method" {
                    assert_eq!(value, "GET");
                }
                if key == "X-Request-Url" {
                    assert_eq!(value, "/abc");
                }
            }
        }
    }
}
