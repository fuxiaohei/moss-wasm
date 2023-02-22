use crate::moss_rpc::moss_rpc_service_client::MossRpcServiceClient;
use crate::moss_rpc::BundleUploadRequest;
use tonic::{metadata::MetadataValue, transport::Channel, Request};

pub async fn upload_bundle() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://127.0.0.1:8679").connect().await?;

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
