tonic::include_proto!("moss");

mod rpc_server;
pub use rpc_server::start;

mod rpc_client;
pub use rpc_client::Client;

mod auth;
