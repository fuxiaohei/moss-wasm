use tonic::{metadata::MetadataValue, service::Interceptor, Request, Status};

/// AUTH_STATIC_TOKEN is a static token for authorization
const AUTH_STATIC_TOKEN: &str = "e20a0453781758a542116380672548449e3a34ef";
/// AUTH_STATIC_HEADER is a static token header
const AUTH_STATIC_HEADER: &str = "authorization";
const AUTH_DYNAMIC_TOKRN_HEADER: &str = "x-moss-token";
const AUTH_DYNAMIC_SECRET_HEADER: &str = "x-moss-signature";
const AUTH_DYNAMIC_ACTION_HEADER: &str = "x-moss-action";

const AUTH_ACTION_CLI_AUTH: &str = "moss_cli_auth";
const AUTH_ACTION_CLI_RPC_CALL: &str = "moss_cli_rpc_call";

/// AuthStaticTokenInterceptor is a interceptor to add static token to request
pub struct AuthStaticTokenInterceptor {}

impl Interceptor for AuthStaticTokenInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        let token: MetadataValue<_> = format!("Bearer {AUTH_STATIC_TOKEN}").parse().unwrap();
        req.metadata_mut().insert(AUTH_STATIC_HEADER, token);
        req.metadata_mut().insert(
            AUTH_DYNAMIC_ACTION_HEADER,
            MetadataValue::from_static(AUTH_ACTION_CLI_AUTH),
        );
        Ok(req)
    }
}

/// AuthDynamicTokenInterceptor is a interceptor to add dynamic token to request
pub struct AuthDynamicTokenInterceptor {
    pub access_token: String,
    pub secret_token: String,
}

impl Interceptor for AuthDynamicTokenInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        let token: MetadataValue<_> = self.access_token.parse().unwrap();
        req.metadata_mut().insert(AUTH_DYNAMIC_TOKRN_HEADER, token);
        let secret: MetadataValue<_> = self.secret_token.parse().unwrap();
        req.metadata_mut()
            .insert(AUTH_DYNAMIC_SECRET_HEADER, secret);
        req.metadata_mut().insert(
            AUTH_DYNAMIC_ACTION_HEADER,
            MetadataValue::from_static(AUTH_ACTION_CLI_RPC_CALL),
        );
        Ok(req)
    }
}

/// verify_auth_token verify auth token
pub async fn verify_auth_token<T>(req: Request<T>) -> Result<Request<T>, Status> {
    // get action name
    let action = match req.metadata().get(AUTH_DYNAMIC_ACTION_HEADER) {
        Some(t) => t,
        _ => return Err(Status::unauthenticated("Invalid rpc action")),
    };

    // cli auth request
    if action == AUTH_ACTION_CLI_AUTH {
        let token = match req.metadata().get(AUTH_STATIC_HEADER) {
            Some(t) => t,
            _ => return Err(Status::unauthenticated("Invalid auth token")),
        };
        let static_token: MetadataValue<_> = format!("Bearer {AUTH_STATIC_TOKEN}").parse().unwrap();
        if token == static_token {
            return Ok(req);
        }
    }

    if action == AUTH_ACTION_CLI_RPC_CALL {
        let token = match req.metadata().get(AUTH_DYNAMIC_TOKRN_HEADER) {
            Some(t) => t,
            _ => return Err(Status::unauthenticated("Access token is required")),
        };
        let secret = match req.metadata().get(AUTH_DYNAMIC_SECRET_HEADER) {
            Some(t) => t,
            _ => return Err(Status::unauthenticated("Signature token is required")),
        };

        let token = token.to_str().unwrap().to_string();
        let token_data = moss_db_service::user::find_by_token(token)
            .await
            .map_err(|_| Status::unauthenticated("Access token is missing"))?;
        if token_data.status != "active" {
            return Err(Status::unauthenticated("Access token is inactive"));
        }
        if moss_db_service::user::is_token_expired(&token_data) {
            return Err(Status::unauthenticated("Access token is expired"));
        }

        let secret_token: MetadataValue<_> = token_data.secret_token.parse().unwrap();
        if secret != secret_token {
            return Err(Status::unauthenticated("Signature token is incorrect"));
        }

        return Ok(req);
    }

    Err(Status::unauthenticated("Invalid rpc action"))
}
