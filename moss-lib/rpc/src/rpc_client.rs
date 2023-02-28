use crate::moss_rpc::moss_rpc_service_client::MossRpcServiceClient;
use crate::moss_rpc::{BundleUploadRequest, TokenRequest, TokenResponse};
use tonic::{metadata::MetadataValue, transport::Channel, Request};
use tracing::{debug, instrument};

/// AUTH_STATIC_TOKEN is a static token for authorization
const AUTH_STATIC_TOKEN: &str = "e20a0453781758a542116380672548449e3a34ef";

pub struct Client {
    addr: String,
}

impl Client {
    pub fn new(addr: String) -> Self {
        Self { addr }
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
        let channel = Channel::from_shared(self.addr)?.connect().await?;
        let token: MetadataValue<_> = format!("Bearer {}", AUTH_STATIC_TOKEN).parse()?;
        let mut client =
            MossRpcServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
                req.metadata_mut().insert("authorization", token.clone());
                req.metadata_mut()
                    .insert("x-auth-method", MetadataValue::from_static("moss_cli_auth_token"));
                Ok(req)
            });
        let request = Request::new(TokenRequest { token: user_token });
        let response = client.verify_token(request).await?;
        debug!("response: {:?}", response);
        Ok(response.into_inner())
    }
}

pub async fn upload_bundle(addr: &'static str) -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static(addr).connect().await?;
    let token: MetadataValue<_> = "Bearer some-secret-token".parse()?;
    let mut client = MossRpcServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("authorization", token.clone());
        Ok(req)
    });

    let request = Request::new(BundleUploadRequest {
        bundle_name: "bundle_name".to_string(),
        bundle_size: 100,
        bundle_md5: "bundle_md5".to_string(),
        content: vec![1, 2, 3, 4],
    });

    let response = client.upload_bundle(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
