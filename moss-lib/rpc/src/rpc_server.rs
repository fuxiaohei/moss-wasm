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
        _request: Request<BundleUploadRequest>,
    ) -> Result<Response<BundleUploadResponse>, Status> {
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
        let token_str = request.into_inner().token;
        match moss_db_service::user_token::find_by_token(token_str).await {
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

/// AUTH_STATIC_TOKEN is a static token for authorization
const AUTH_STATIC_TOKEN: &str = "e20a0453781758a542116380672548449e3a34ef";

/// auth_middleware is middleware to check authorization
fn auth_middleware(req: Request<()>) -> Result<Request<()>, Status> {
    let real_token = match req.metadata().get("authorization") {
        Some(t) => t,
        _ => return Err(Status::unauthenticated("Invalid auth token")),
    };

    let auth_method = req.metadata().get("x-auth-method");
    // check auth_method if exist, use inner auth token
    if auth_method.is_some() {
        let auth_method = auth_method.unwrap().to_str().unwrap();
        if auth_method == "moss_cli_auth_token" {
            return Ok(req);
        }
        let static_token: MetadataValue<_> =
            format!("Bearer {}", AUTH_STATIC_TOKEN).parse().unwrap();
        if real_token == static_token {
            return Ok(req);
        }
    }

    let token: MetadataValue<_> = "Bearer some-secret-token".parse().unwrap();
    if real_token == token {
        return Ok(req);
    }
    Err(Status::unauthenticated("Invalid auth token"))
}

/// start startes rpc server
pub async fn start(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let rpc_impl = MossRpcImpl::default();
    let svc = MossRpcServiceServer::with_interceptor(rpc_impl, auth_middleware);

    println!("MossRpcServer listening on {}", addr);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
