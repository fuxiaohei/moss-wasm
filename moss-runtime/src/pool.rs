use crate::worker::Worker;
use anyhow::Result;
use async_trait::async_trait;
use deadpool::managed;
use tracing::debug;

#[derive(Debug)]
pub struct Manager {
    path: String,
}

impl Manager {
    pub fn new(path: &str) -> Self {
        Self {
            path: String::from(path),
        }
    }
}

#[async_trait]
impl managed::Manager for Manager {
    type Type = Worker;
    type Error = anyhow::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        println!("create worker: {}", self.path);
        Ok(Worker::new(&self.path).await?)
    }

    async fn recycle(&self, _obj: &mut Self::Type) -> managed::RecycleResult<Self::Error> {
        Ok(())
    }
}

pub type WorkerPool = managed::Pool<Manager>;

/// create a pool
pub fn create(path: &str) -> Result<WorkerPool> {
    let mgr = Manager::new(path);
    Ok(managed::Pool::builder(mgr).build().unwrap())
}
