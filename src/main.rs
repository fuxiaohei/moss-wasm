use moss_host_call::http_impl;
use moss_runtime::compiler;
use moss_runtime::pool;

#[tokio::main]
async fn main() {
    let mut name = std::env::args().nth(1).unwrap();
    name = name.replace('-', "_");
    println!("Run name: {name}");

    let target = format!("target/wasm32-wasi/release/{name}.wasm");
    let output = format!("target/wasm32-wasi/release/{name}.component.wasm");

    compiler::convert_component(&target, Some(output.to_string()));
    println!("Run component: {output}");

    let worker_pool = pool::create(&output).unwrap();
    let status = worker_pool.status();
    println!("Pool status, {status:?}");

    let mut worker = worker_pool.get().await.unwrap();
    let worker = worker.as_mut();

    let headers = vec![("Content-Type", "application/json")];
    let req = http_impl::http_handler::Request {
        method: "GET",
        uri: "/abc",
        headers: &headers,
        body: None,
    };

    let resp = worker.execute(req).await.unwrap();
    println!("-----\nstatus, {:?}", resp.status);
    for (key, value) in resp.headers {
        println!("header, {key}: {value}");
    }
    println!("body, {:?}", resp.body.unwrap().len());
}
