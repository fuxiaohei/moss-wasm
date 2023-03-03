use crate::auth::{AuthDynamicTokenInterceptor, AuthStaticTokenInterceptor};
use crate::moss_rpc_service_client::MossRpcServiceClient;
use crate::{BundleUploadRequest, TokenRequest, TokenResponse};
use tonic::{codegen::InterceptedService, transport::Channel, Request};
use tracing::{debug, instrument};

pub struct Client {
    addr: String,
    access_token: String,
    secret_token: String,
}

impl Client {
    pub fn new(addr: String, access_token: String, secret_token: String) -> Self {
        Self {
            addr,
            access_token,
            secret_token,
        }
    }

    async fn create_static_client(
        self,
    ) -> Result<
        MossRpcServiceClient<InterceptedService<Channel, AuthStaticTokenInterceptor>>,
        Box<dyn std::error::Error>,
    > {
        let channel = Channel::from_shared(self.addr)?.connect().await?;
        let client = MossRpcServiceClient::with_interceptor(channel, AuthStaticTokenInterceptor {});
        Ok(client)
    }

    async fn create_token_client(
        self,
    ) -> Result<
        MossRpcServiceClient<InterceptedService<Channel, AuthDynamicTokenInterceptor>>,
        Box<dyn std::error::Error>,
    > {
        let channel = Channel::from_shared(self.addr)?.connect().await?;
        let client = MossRpcServiceClient::with_interceptor(
            channel,
            AuthDynamicTokenInterceptor {
                access_token: self.access_token.clone(),
                secret_token: self.secret_token.clone(),
            },
        );
        Ok(client)
    }

    #[instrument(
        skip_all,
        name = "[Rpc]",
        level = "debug",
        fields(method = "auth_token")
    )]
    pub async fn auth_token(
        self,
        user_token: String,
    ) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        let mut client = self.create_static_client().await?;
        let request = Request::new(TokenRequest { token: user_token });
        let response = client.verify_token(request).await?;
        debug!("[auth_token] response={:?}", response);
        Ok(response.into_inner())
    }

    #[instrument(
        skip_all,
        name = "[Rpc]",
        level = "debug",
        fields(method = "upload_function")
    )]
    pub async fn upload_function(
        self,
        bundle_req: BundleUploadRequest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut client = self.create_token_client().await?;
        let request = Request::new(bundle_req);
        let response = client.upload_bundle(request).await?;
        debug!("response={response:?}");
        Ok(())
    }
}
