use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// DEFAULT_METADATA_FILE is the default metadata file name
pub const DEFAULT_METADATA_FILE: &str = "metadata.toml";

/// Metadata is the Metadata struct
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub manifest: String,
    pub name: String,
    pub description: String,
    pub authors: Vec<String>,
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<MetadataBuild>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deploy: Option<MetadataDeploy>,
}

/// MetadataBuild is the build section of the Metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetadataBuild {
    pub rust_target_dir: Option<String>,
}

// MetadataDeploy is the deploy section of the Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataDeploy {
    pub trigger: String,
    pub route_base: Option<String>,
}

impl Default for MetadataDeploy {
    fn default() -> Self {
        Self {
            trigger: "http".to_string(),
            route_base: Some("/*path".to_string()),
        }
    }
}

impl Metadata {
    /// read Metadata from toml file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut manifest: Metadata = toml::from_str(&content)?;

        // fill value to default for Option<T>
        if manifest.build.is_none() {
            manifest.build = Some(MetadataBuild::default());
        }
        if manifest.deploy.is_none() {
            manifest.deploy = Some(MetadataDeploy::default());
        }

        Ok(manifest)
    }

    /// read Metadata from binary
    pub fn from_binary(data: &[u8]) -> Result<Self> {
        let manifest: Metadata = toml::from_str(std::str::from_utf8(data)?)?;
        Ok(manifest)
    }

    /// write Metadata to toml file
    pub fn to_file(&self, path: &str) -> Result<()> {
        let content = toml::to_string(&self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// get arch from metadata
    pub fn get_arch(&self) -> String {
        "wasm32-wasi".to_string()
    }

    /// is wasi
    pub fn is_wasi(&self) -> bool {
        self.get_arch() == "wasm32-wasi"
    }

    /// get compiled target
    pub fn get_target(&self) -> String {
        let target = self
            .build
            .clone()
            .unwrap_or_default()
            .rust_target_dir
            .unwrap_or_else(|| "target".to_string());
        let arch = self.get_arch();
        let target_dir = Path::new(&target).join(arch).join("release");
        let name = self.name.replace('-', "_") + ".wasm";
        target_dir.join(name).to_str().unwrap().to_string()
    }

    /// get output file
    pub fn get_output(&self) -> String {
        self.get_target().replace(".wasm", ".component.wasm")
    }

    /// get src directory name
    pub fn get_src_dir(&self) -> String {
        if self.language == "js" {
            return "dist/".to_string();
        }
        "src/".to_string()
    }

    /// get route base
    pub fn get_route_base(&self) -> String {
        self.deploy
            .clone()
            .unwrap_or_default()
            .route_base
            .unwrap_or_else(|| "/*path".to_string())
    }
}

/// DEFAULT_ENV_FILE is the default env file name
pub const DEFAULT_ENV_FILE: &str = ".moss_cli.env";

/// get metadata env file from home path
pub fn get_metadata_env_file() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    Path::new(&home)
        .join(DEFAULT_ENV_FILE)
        .to_str()
        .unwrap()
        .to_string()
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetadataEnv {
    pub api_key: String,
    pub api_secret: String,
    pub api_secret_expires: u64,
    pub api_host: String,
    pub created_at: u64,
}

impl MetadataEnv {
    pub fn to_file(&self, path: &str) -> Result<()> {
        // use bincode
        let content = bincode::serialize(&self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read(path)?;
        let env: MetadataEnv = bincode::deserialize(&content)?;
        Ok(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// test manifest from_file
    #[test]
    fn from_file() {
        let manifest = Metadata::from_file("../tests/data/metadata.toml").unwrap();
        assert_eq!(manifest.manifest, "v1");
        assert_eq!(manifest.name, "rust-basic");
        assert_eq!(manifest.description, "example rust project");
        assert_eq!(manifest.authors, vec!["leaf"]);
        assert_eq!(manifest.language, "rust");
        assert_eq!(
            manifest.build.as_ref().unwrap().rust_target_dir,
            Some("./target".to_string())
        );
    }

    /// test manifest to file
    #[test]
    fn to_file() {
        let manifest = Metadata::from_file("../tests/data/metadata.toml").unwrap();
        manifest.to_file("../tests/data/metadata2.toml").unwrap();
        let manifest2 = Metadata::from_file("../tests/data/metadata2.toml").unwrap();
        assert_eq!(manifest.manifest, manifest2.manifest);
        assert_eq!(manifest.name, manifest2.name);
        assert_eq!(manifest.description, manifest2.description);
        assert_eq!(manifest.authors, manifest2.authors);
        assert_eq!(manifest.language, manifest2.language);
        assert_eq!(
            manifest.build.as_ref().unwrap().rust_target_dir,
            manifest2.build.as_ref().unwrap().rust_target_dir
        );
        std::fs::remove_file("../tests/data/metadata2.toml").unwrap();
    }

    /// test metadata env
    #[test]
    fn metadata_env() {
        let env_data = MetadataEnv {
            api_key: "api_key".to_string(),
            api_secret: "api_secret".to_string(),
            api_secret_expires: 123456789,
            api_host: "api_host".to_string(),
            created_at: 123456789,
        };
        env_data.to_file("../tests/data/metadata.env").unwrap();
        let env_data2 = MetadataEnv::from_file("../tests/data/metadata.env").unwrap();
        assert_eq!(env_data.api_key, env_data2.api_key);
        assert_eq!(env_data.api_secret, env_data2.api_secret);
        assert_eq!(env_data.api_secret_expires, env_data2.api_secret_expires);
        assert_eq!(env_data.api_host, env_data2.api_host);
        assert_eq!(env_data.created_at, env_data2.created_at);

        std::fs::remove_file("../tests/data/metadata.env").unwrap();
    }
}
