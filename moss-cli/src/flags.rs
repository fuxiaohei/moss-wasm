use clap::Args;
use std::net::SocketAddr;
use tracing::{debug, debug_span, Instrument};

#[derive(Args, Debug)]
pub struct Init {
    /// The name of the project
    pub name: String,
    /// The template to use
    #[clap(long, default_value("rust-basic"))]
    pub template: Option<String>,
}

impl Init {
    pub async fn run(&self) {
        println!("Init: {:?}", self);
    }
}

#[derive(Args, Debug)]
pub struct Build {
    /// Set js engine wasm file
    #[clap(long)]
    pub js_engine: Option<String>,
}

impl Build {
    pub async fn run(&self) {
        println!("Build: {:?}", self);
    }
}

#[derive(Args, Debug)]
pub struct Serve {
    /// The port to listen on
    #[clap(long, default_value("127.0.0.1:8678"))]
    pub addr: Option<SocketAddr>,
}

impl Serve {
    pub async fn run(&self) {
        debug!("Serve: {:?}", self);
        crate::server::start(
            self.addr.unwrap(),
            "target/wasm32-wasi/release/rust_basic.component.wasm",
        )
        .instrument(debug_span!("[Http]"))
        .await;
    }
}
