use moss_host_call::http_impl;
use moss_runtime::compiler;
use moss_runtime::worker::Worker;

#[tokio::main]
async fn main() {
    let mut name = std::env::args().nth(1).unwrap();
    name = name.replace("-", "_");
    println!("Run name: {}", name);

    let target = format!("target/wasm32-wasi/release/{}.wasm", name);
    let output = format!("target/wasm32-wasi/release/{}.component.wasm", name);

    compiler::convert_component(&target, Some(output.to_string()));
    println!("Run component: {}", output);

    let mut worker = Worker::new(&output).await.unwrap();

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
        println!("header, {}: {}", key, value);
    }
    println!("body, {:?}", resp.body.unwrap().len());
}
