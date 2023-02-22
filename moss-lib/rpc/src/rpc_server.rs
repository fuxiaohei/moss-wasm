use crate::moss_rpc::moss_rpc_service_server::{MossRpcService, MossRpcServiceServer};
use crate::moss_rpc::{BundleUploadRequest, BundleUploadResponse};
use std::net::SocketAddr;
use tonic::{metadata::MetadataValue, transport::Server, Request, Response, Status};

#[derive(Default)]
pub struct MossRpcImpl {}

#[tonic::async_trait]
impl MossRpcService for MossRpcImpl {
    async fn upload_bundle(
        &self,
        request: Request<BundleUploadRequest>,
    ) -> Result<Response<BundleUploadResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let resp = BundleUploadResponse {
            status_code: 1,
            message: "response ok".to_string(),
        };
        Ok(Response::new(resp))
    }
}

/// auth_middleware is middleware to check authorization
fn auth_middleware(req: Request<()>) -> Result<Request<()>, Status> {
    let token: MetadataValue<_> = "Bearer some-secret-token".parse().unwrap();

    match req.metadata().get("authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("Invalid auth token")),
    }
}

/// start startes rpc server
pub async fn start(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let rpc_impl = MossRpcImpl::default();
    let svc = MossRpcServiceServer::with_interceptor(rpc_impl, auth_middleware);

    println!("MossRpcServer listening on {}", addr);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
