use crate::moss_rpc::moss_rpc_service_server::{MossRpcService, MossRpcServiceServer};
use crate::moss_rpc::{BundleUploadRequest, BundleUploadResponse, TokenRequest, TokenResponse};
use std::net::SocketAddr;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

#[derive(Default)]
pub struct MossRpcImpl {}

#[tonic::async_trait]
impl MossRpcService for MossRpcImpl {
    async fn upload_bundle(
        &self,
        req: Request<BundleUploadRequest>,
    ) -> Result<Response<BundleUploadResponse>, Status> {
        crate::auth::verify_auth_token(req).await?;

        let resp = BundleUploadResponse {
            status_code: 1,
            message: "response ok".to_string(),
        };
        Ok(Response::new(resp))
    }

    async fn verify_token(
        &self,
        request: Request<TokenRequest>,
    ) -> Result<Response<TokenResponse>, Status> {
        let token_str = request.into_inner().token;
        match moss_db_service::user::find_by_token(token_str).await {
            Ok(token) => {
                let resp = TokenResponse {
                    access_token: token.access_token,
                    secret_token: token.secret_token,
                    expiration: token.expired_at,
                };
                Ok(Response::new(resp))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}

/// start startes rpc server
pub async fn start(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let rpc_impl = MossRpcImpl::default();
    let svc = MossRpcServiceServer::new(rpc_impl);
    info!("MossRpcServer listening on {addr}");
    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
