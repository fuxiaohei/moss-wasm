use clap::Parser;
use std::net::SocketAddr;
use std::path::Path;
use tracing::error;

#[derive(Parser, Debug)]
struct CliArgs {
    #[clap(long, default_value("moss-server.toml"))]
    pub config: String,
    #[clap(long, default_value("127.0.0.1:8679"))]
    pub addr: SocketAddr,
}

fn main() {
    moss_lib::tracing::init_tracing();

    let args = CliArgs::parse();
    println!("{args:?}");

    // load config file from args, check file exist
    if !Path::new(&args.config).exists() {
        error!("Config file {} not found", &args.config);
        return;
    }
}
