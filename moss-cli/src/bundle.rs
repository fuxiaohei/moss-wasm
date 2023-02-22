use std::io::Write;

use anyhow::Result;
use moss_lib::metadata::DEFAULT_METADATA_FILE;
use std::path::Path;

pub fn build(output: &str, metadata: &str) -> Result<String> {
    let output_path = Path::new(output);
    let output_path = output_path.file_name().unwrap().to_str().unwrap();
    let bundle_file = output_path.replace(".wasm", ".zip");

    let file = std::fs::File::create(&bundle_file).unwrap();
    let mut zip = zip::ZipWriter::new(file);

    // add wasm file
    zip.start_file(output_path, Default::default())?;
    zip.write_all(&std::fs::read(output)?)?;

    // add metadata file
    zip.start_file(DEFAULT_METADATA_FILE, Default::default())?;
    zip.write_all(&std::fs::read(metadata)?)?;

    zip.flush()?;
    Ok(bundle_file)
}