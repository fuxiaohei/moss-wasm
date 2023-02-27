use crate::{bundle, embed};
use clap::Args;
use moss_lib::metadata::{Metadata, DEFAULT_METADATA_FILE};
use moss_rpc::rpc_client;
use moss_runtime::compiler;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, debug_span, error, info, Instrument};

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
        debug!("Init: {self:?}");
        // create dir by name
        if !Path::new(&self.name).exists() {
            std::fs::create_dir(&self.name).unwrap();
            info!("Created dir: {}", &self.name)
        }
        // create metadata
        let meta = self.create_metadata();

        // create project
        self.create_project(meta);

        info!("Created project {} success", &self.name);
    }

    fn create_metadata(&self) -> Metadata {
        let metadata_file =
            PathBuf::from(&self.template.as_ref().unwrap()).join(DEFAULT_METADATA_FILE);
        let meta = embed::TemplateAssets::get(metadata_file.to_str().unwrap());
        if meta.is_none() {
            panic!("Template {} is not valid", &self.template.as_ref().unwrap());
        }
        let mut meta = Metadata::from_binary(&meta.unwrap().data).unwrap();
        meta.name = self.name.clone();
        meta.build = None;
        let metadata_file = PathBuf::from(&self.name).join(DEFAULT_METADATA_FILE);
        meta.to_file(metadata_file.to_str().unwrap()).unwrap();
        info!("Created metadata: {:?}", metadata_file);
        meta
    }

    fn create_cargo_toml(&self) {
        let template = self.template.as_ref().unwrap();
        let name = self.name.as_str();

        let toml_file = PathBuf::from(template).join("Cargo.toml");
        let toml_data = embed::TemplateAssets::get(toml_file.to_str().unwrap());
        if toml_data.is_none() {
            panic!(
                "Template {} is not valid with rust Cargo.toml",
                &self.template.as_ref().unwrap()
            );
        }
        let target_file = PathBuf::from(&self.name).join("Cargo.toml");

        // replace cargo toml to correct deps
        let mut content = std::str::from_utf8(&toml_data.unwrap().data)
            .unwrap()
            .to_string();
        content = content.replace(template, name);
        content = content.replace(
            "path = \"../../moss-sdk\"",
            "git = \"https://github.com/fuxiaohei/moss-wasm\"",
        );
        std::fs::write(target_file.to_str().unwrap(), content).unwrap();

        info!("Created Cargo.toml: {:?}", target_file);
    }

    fn create_project(&self, meta: Metadata) {
        // if rust project, copy Cargo.toml
        if meta.language == "rust" {
            self.create_cargo_toml();
        }

        // create src dir
        let src_dir = Path::new(&self.name).join("src");
        std::fs::create_dir_all(src_dir.parent().unwrap()).unwrap();

        // copy src files
        let tpl_dir = Path::new(&self.template.as_ref().unwrap()).join("src");
        embed::TemplateAssets::iter().for_each(|t| {
            if t.starts_with(tpl_dir.to_str().unwrap()) {
                let src_path = Path::new(t.as_ref())
                    .strip_prefix(tpl_dir.to_str().unwrap())
                    .unwrap();
                let file = embed::TemplateAssets::get(t.as_ref()).unwrap();
                let content = std::str::from_utf8(&file.data).unwrap().to_string();
                let target_path = src_dir.join(src_path);
                debug!("Created src: {:?}, {:?}", src_path, target_path);
                std::fs::create_dir_all(target_path.parent().unwrap()).unwrap();
                std::fs::write(target_path, content).unwrap();
            }
        });
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
        debug!("Build: {self:?}");

        // read metadata from file
        let meta =
            Metadata::from_file(DEFAULT_METADATA_FILE).expect("Project metadata.toml not found");
        let arch = meta.get_arch();
        info!("Build arch: {}", arch);

        let target = meta.get_target();
        info!("Build target: {}", target);

        // call cargo to build wasm
        match meta.language.as_str() {
            "rust" => {
                compiler::compile_rust(&arch, &target).expect("Build failed");
            }
            "js" => {
                compiler::compile_js(&target, "src/index.js", self.js_engine.clone())
                    .expect("Build failed");
            }
            _ => {
                panic!("Unsupported language: {}", meta.language);
            }
        }

        // convert wasm module to component
        let output = meta.get_output();
        compiler::convert_component(&target, Some(output)).expect("Convert failed");
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
        debug!("Serve: {self:?}");

        let meta =
            Metadata::from_file(DEFAULT_METADATA_FILE).expect("Project metadata.toml not found");
        debug!("Metadata: {meta:?}");

        let output = meta.get_output();
        if !Path::new(&output).exists() {
            panic!("Component {} not found, please build first", &output);
        }
        info!("Serve component: {}", &output);

        if meta.is_wasi() {
            info!("Enable wasm32-wasi");
        }

        crate::server::start(self.addr.unwrap(), meta)
            .instrument(debug_span!("[Http]"))
            .await;
    }
}

#[derive(Args, Debug)]
pub struct Deploy {}

impl Deploy {
    pub async fn run(&self) {
        debug!("Deploy: {self:?}");

        let meta =
            Metadata::from_file(DEFAULT_METADATA_FILE).expect("Project metadata.toml not found");
        debug!("Metadata: {meta:?}");

        // make sure output file is generated
        let output = meta.get_output();
        if !Path::new(&output).exists() {
            panic!("Component {} not found, please build first", &output);
        }
        info!("Find component: {}", &output);

        // generate an bundled file
        let bundle_file = bundle::build(&output, DEFAULT_METADATA_FILE).unwrap();

        // upload bundle
        bundle::deploy(&bundle_file).await.unwrap();
    }
}

#[derive(Args, Debug)]
pub struct Auth {
    /// The user token
    pub user_token: String,
    /// Cloud api address
    #[clap(long, default_value("http://127.0.0.1:8679"))]
    pub cloud_api: Option<String>,
}

impl Auth {
    pub async fn run(&self) {
        debug!("Auth: {self:?}");

        let cloud_api = self.cloud_api.as_ref().unwrap().clone();
        info!("Connect to {}", cloud_api);

        let client = rpc_client::Client::new(cloud_api.clone());
        let response = match client.auth_token(self.user_token.clone()).await {
            Ok(response) => response,
            Err(e) => {
                error!("Auth failed: {}", e);
                return;
            }
        };
        // if expiration > 0, need to check expiration
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if response.expiration > 0 {
            if now > response.expiration.try_into().unwrap() {
                error!("Auth failed: token expired");
                return;
            }
        }

        // get env file
        let env_file = moss_lib::metadata::get_metadata_env_file();
        debug!("Env file: {:?}", env_file);

        // write env file
        let env_data = moss_lib::metadata::MetadataEnv {
            api_host: cloud_api,
            api_key: self.user_token.clone(),
            api_secret: response.secret_token,
            api_secret_expires: response.expiration.try_into().unwrap(),
            created_at: now,
        };
        env_data.to_file(&env_file).unwrap();

        info!("Auth success!");
    }
}
