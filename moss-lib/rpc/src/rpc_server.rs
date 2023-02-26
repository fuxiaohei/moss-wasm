use crate::moss_rpc::moss_rpc_service_server::{MossRpcService, MossRpcServiceServer};
use crate::moss_rpc::{BundleUploadRequest, BundleUploadResponse, TokenRequest, TokenResponse};
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

    async fn get_token(
        &self,
        request: Request<TokenRequest>,
    ) -> Result<Response<TokenResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let token_str = request.into_inner().token;
        match moss_service::get_user_token(token_str).await {
            Ok(token) => {
                let resp = TokenResponse {
                    access_token: token.access_token,
                    secret_token: token.secret_token,
                    expiration: 3600,
                };
                Ok(Response::new(resp))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
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
