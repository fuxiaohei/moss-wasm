use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub driver: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fs: Option<FsConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FsConfig {
    pub directory: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            driver: "fs".to_string(),
            fs: Some(FsConfig {
                directory: "./data/moss-serverless/".to_string(),
            }),
        }
    }
}
