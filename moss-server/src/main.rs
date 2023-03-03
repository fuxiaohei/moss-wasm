use clap::Parser;
use std::path::Path;
use tracing::{debug, error};

mod config;

#[derive(Parser, Debug)]
struct CliArgs {
    #[clap(long, default_value("moss-server.toml"))]
    pub config: String,
}

#[tokio::main]
async fn main() {
    moss_lib::tracing::init_tracing();

    let args = CliArgs::parse();
    println!("{args:?}");

    // load config file from args, check file exist
    if !Path::new(&args.config).exists() {
        error!("Config file {} not found", &args.config);
        return;
    }

    // read config file
    let config = config::Config::from_file(&args.config).unwrap();
    debug!("read config: {config:?}");

    // init database
    moss_db_service::init_db(&config.db).await.unwrap();

    // start rpc server
    moss_rpc_service::start(config.http.addr.parse().unwrap())
        .await
        .unwrap();
}
