use clap::Parser;
use moss_host_call::http_impl;
use moss_runtime::compiler;
use moss_runtime::pool;

#[derive(Parser, Debug)]
struct CliArgs {
    name: String,
    #[clap(long, default_value("true"))]
    wasi: Option<bool>,
}

#[tokio::main]
async fn main() {
    let start_time = tokio::time::Instant::now();

    let args = CliArgs::parse();

    let name = args.name.replace('-', "_");
    println!("Run name: {name}");

    let arch = if args.wasi.unwrap() {
        "wasm32-wasi"
    } else {
        "wasm32-unknown-unknown"
    };
    println!("Run arch: {arch}");

    let target = format!("target/{arch}/release/{name}.wasm");
    let output = format!("target/{arch}/release/{name}.component.wasm");

    compiler::convert_component(&target, Some(output.to_string())).unwrap();
    println!("Run component: {output}");

    let worker_pool = pool::create(&output, arch == "wasm32-wasi").unwrap();
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

    let resp = worker.handle_request(req).await.unwrap();
    println!("-----\nstatus, {:?}", resp.status);
    for (key, value) in resp.headers {
        println!("header, {key}: {value}");
    }
    println!("body, {:?}", resp.body.unwrap().len());
    println!("elapsed, {:?}", start_time.elapsed());
    println!("-----")
    
}
