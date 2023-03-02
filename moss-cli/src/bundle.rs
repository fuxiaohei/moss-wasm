use anyhow::Result;
use md5::{Digest, Md5};
use moss_lib::metadata::{MetadataEnv, DEFAULT_METADATA_FILE};
use moss_rpc::moss_rpc::BundleUploadRequest;
use moss_rpc::rpc_client;
use std::io::Write;
use std::path::Path;
use tracing::{debug, error, info};
use walkdir::WalkDir;

/// Bundle is a struct to store bundle file
pub struct Bundle {
    pub name: String,
    pub size: u64,
    pub md5: String,
    pub content: Vec<u8>,
}

pub fn build(output: &str, metadata: &str, src_dir: &str) -> Result<Bundle> {
    let output_path = Path::new(output);
    let output_path = output_path.file_name().unwrap().to_str().unwrap();
    let bundle_file = output_path.replace(".wasm", ".zip");

    let file = std::fs::File::create(&bundle_file).unwrap();
    let mut zip = zip::ZipWriter::new(file);

    // add wasm file
    debug!("[bundle] add wasm file: {}", output_path);
    zip.start_file(output_path, Default::default())?;
    zip.write_all(&std::fs::read(output)?)?;

    // add metadata file
    debug!("[bundle] add metadata file: {}", metadata);
    zip.start_file(DEFAULT_METADATA_FILE, Default::default())?;
    zip.write_all(&std::fs::read(metadata)?)?;

    // add src directory
    let src_dir = Path::new(src_dir);
    let walkdir = WalkDir::new(src_dir);
    for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let path = path.to_str().unwrap();
        debug!("[bundle] add src file: {}", path);
        zip.start_file(path, Default::default())?;
        zip.write_all(&std::fs::read(path)?)?;
    }

    zip.flush().expect("flush zip file");

    // show bundle size
    let bundle_size = std::fs::metadata(&bundle_file)?.len();
    info!(
        "bundle size: {:.2} MB",
        bundle_size as f64 / 1024.0 / 1024.0
    );

    let mut hasher = Md5::new();
    let bundle_content = std::fs::read(&bundle_file)?;
    hasher.update(&bundle_content);
    let bundle_hash = format!("{:x}", hasher.finalize());

    Ok(Bundle {
        name: bundle_file,
        size: bundle_size,
        md5: bundle_hash,
        content: bundle_content,
    })
}

pub async fn deploy(env: &MetadataEnv, bundle: &Bundle) -> Result<()> {
    // create client
    let client = rpc_client::Client::new(
        env.api_host.clone(),
        env.api_key.clone(),
        env.api_secret.clone(),
    );
    let req = BundleUploadRequest {
        bundle_name: bundle.name.clone(),
        bundle_size: bundle.size as i64,
        bundle_md5: bundle.md5.clone(),
        content: bundle.content.clone(),
    };
    match client.upload_function(req).await {
        Ok(response) => response,
        Err(e) => {
            error!("deploy error: {}", e);
            return Ok(());
        }
    };
    info!("Deploy OK");

    Ok(())
}
