use clap::Parser;

mod embed;
mod flags;
mod server;

///  moss-cli command line
#[derive(Parser)]
#[clap(
    name = "moss-cli",
    version = moss_lib::version::get_version(),
)]

enum MossCli {
    /// Init creates a new project
    Init(flags::Init),
    /// Build compiles the project
    Build(flags::Build),
    /// Serve runs the project
    Serve(flags::Serve),
}

#[tokio::main]
async fn main() {
    moss_lib::tracing::init_tracing();

    let args = MossCli::parse();
    match args {
        MossCli::Init(cmd) => cmd.run().await,
        MossCli::Build(cmd) => cmd.run().await,
        MossCli::Serve(cmd) => cmd.run().await,
    }
}
