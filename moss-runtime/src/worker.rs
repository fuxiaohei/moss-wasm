use crate::context::Context;
use anyhow::Result;
use moss_host_call::fetch_impl;
use moss_host_call::http_impl;
use moss_host_call::kv_impl;
use wasmtime::component::{Component, Instance, InstancePre, Linker};
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
    component: Component,
    is_wasi: bool,

    // use for wasi
    instance_pre: Option<InstancePre<Context>>,

    // use for wasm32
    instance: Option<Instance>,
    store: Option<Store<Context>>,
    exports: Option<http_impl::HttpHandler>,
    is_trapped: bool,
}

impl Worker {
    pub async fn new(path: &str, is_wasi: bool) -> Result<Self> {
        // create component
        let config = create_wasmtime_config();
        let engine = Engine::new(&config)?;
        let component = Component::from_file(&engine, path)?;

        let mut worker = Self {
            _path: path.to_string(),
            engine,
            component,
            instance_pre: None,
            is_wasi,
            instance: None,
            store: None,
            exports: None,
            is_trapped: false,
        };

        if worker.is_wasi {
            worker.create_instance_pre()?;
        } else {
            worker.create_instance().await?;
        }

        Ok(worker)
    }

    async fn create_instance(&mut self) -> Result<()> {
        // create linker
        let mut linker: Linker<Context> = Linker::new(&self.engine);
        wasi_host::add_to_linker(&mut linker, Context::wasi)?;
        fetch_impl::http_fetch::add_to_linker(&mut linker, Context::fetch_impl)?;
        kv_impl::kv_storage::add_to_linker(&mut linker, Context::kv_storage)?;

        // create store
        let mut store = Store::new(&self.engine, Context::new(None));
        let (exports, instance) =
            http_impl::HttpHandler::instantiate_async(&mut store, &self.component, &linker).await?;
        self.is_trapped = false;
        self.store = Some(store);
        self.exports = Some(exports);
        self.instance = Some(instance);
        Ok(())
    }

    fn create_instance_pre(&mut self) -> Result<()> {
        // create linker
        let mut linker: Linker<Context> = Linker::new(&self.engine);
        wasi_host::add_to_linker(&mut linker, Context::wasi)?;
        fetch_impl::http_fetch::add_to_linker(&mut linker, Context::fetch_impl)?;
        kv_impl::kv_storage::add_to_linker(&mut linker, Context::kv_storage)?;

        // create instance_pre
        let instance_pre = linker.instantiate_pre(&self.component)?;
        self.instance_pre = Some(instance_pre);
        Ok(())
    }
    pub async fn handle_request(
        &mut self,
        req: http_impl::http_handler::Request<'_>,
    ) -> Result<http_impl::http_handler::Response> {
        if self.is_wasi {
            return self.execute_wasi(req).await;
        }
        return self.execute_wasm32(req).await;
    }
    pub async fn execute_wasm32(
        &mut self,
        req: http_impl::http_handler::Request<'_>,
    ) -> Result<http_impl::http_handler::Response> {
        if self.is_trapped {
            self.create_instance().await?;
        }
        let store = self.store.as_mut().unwrap();
        let exports = self.exports.as_mut().unwrap();
        let resp = exports
            .http_handler()
            .call_handle_request(store, req)
            .await?;
        Ok(resp)
    }
    pub async fn execute_wasi(
        &mut self,
        req: http_impl::http_handler::Request<'_>,
    ) -> Result<http_impl::http_handler::Response> {
        // create store
        let mut store = Store::new(&self.engine, Context::new(None));

        // get exports and call handle_request
        let instance_pre = self.instance_pre.as_ref().unwrap();
        let (exports, _instance) =
            http_impl::HttpHandler::instantiate_pre(&mut store, instance_pre).await?;
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
    async fn run_wasm32() {
        let wasm_file = "../tests/data/rust_basic.component.wasm";
        let mut worker = Worker::new(wasm_file, true).await.unwrap();

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
