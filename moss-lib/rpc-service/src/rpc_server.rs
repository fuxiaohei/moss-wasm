use crate::moss_rpc_service_server::{MossRpcService, MossRpcServiceServer};
use crate::{BundleUploadRequest, BundleUploadResponse, TokenRequest, TokenResponse};
use moss_db_service::entity::function_info::Model as FunctionInfoModel;
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
        let token_model = crate::auth::verify_rpc_call_token(&req).await?;
        let req = req.into_inner();
        let now = chrono::Utc::now();
        let function_info = FunctionInfoModel {
            id: 0,
            user_id: token_model.user_id,
            name: req.bundle_name,
            uuid: "uuid".to_string(),
            resource: 1,
            status: "active".to_string(),
            function_type: "rust".to_string(),
            storage_path: "/tmp".to_string(),
            storage_size: req.bundle_size as i32,
            storage_md5: req.bundle_md5,
            created_at: now,
            deleted_at: now,
        };
        let model = moss_db_service::function::upsert_info(function_info)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        info!("function info: {:?}", model);

        let resp = BundleUploadResponse {
            status_code: 1,
            message: "response ok".to_string(),
        };
        Ok(Response::new(resp))
    }

    async fn create_token(
        &self,
        request: Request<TokenRequest>,
    ) -> Result<Response<TokenResponse>, Status> {
        crate::auth::verify_auth_token(&request).await?;

        let token_str = request.into_inner().token;
        match moss_db_service::user_token::get(token_str, "moss-cli").await {
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
