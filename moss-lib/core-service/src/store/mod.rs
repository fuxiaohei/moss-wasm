use anyhow::{anyhow, Ok, Result};
use once_cell::sync::OnceCell;
use opendal::services::Fs;
use opendal::Operator;
use tracing::debug;

pub mod config;

pub static STORE: OnceCell<Operator> = OnceCell::new();

/// init_store initializes function store
pub fn init_store(cfg: &config::Config) -> Result<()> {
    debug!("init function store: {cfg:?}");
    if cfg.driver == "fs" {
        if cfg.fs.is_none() {
            return Err(anyhow!("fs config is required"));
        }
        let dir = &cfg.fs.as_ref().unwrap().directory;
        std::fs::create_dir_all(dir)?;
        let mut builder = Fs::default();
        builder.root(dir);

        let op: Operator = Operator::create(builder)?.finish();
        STORE.set(op).unwrap();
        return Ok(());
    }

    Err(anyhow!("unsupported store driver: {}", cfg.driver))
}
