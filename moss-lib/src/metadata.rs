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
}

/// MetadataBuild is the build section of the Metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetadataBuild {
    pub rust_target_dir: Option<String>,
}

impl Metadata {
    /// read Metadata from toml file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let manifest: Metadata = toml::from_str(&content)?;
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
}

#[cfg(test)]
mod tests {
    use super::Metadata;
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
}
