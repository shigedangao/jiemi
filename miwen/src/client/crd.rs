use gen::crd::DecryptorSpec;
use tonic::Request;

use crate::err::Error;
use self::proto::{
    crd_service_client::CrdServiceClient,
    Payload
};

mod proto {
    tonic::include_proto!("crd");
}

impl Payload {
    /// Create a new Payload
    /// 
    /// # Arguments
    /// * `spec` - &DecryptorSpec
    fn new(spec: &DecryptorSpec) -> Self {
        let repository = spec.source.repository.url.to_owned();
        let file_to_decrypt = spec.source.file_to_decrypt.to_owned();
        let sops_file_path = spec.source.sops_path.to_owned();

        Payload {
            file_to_decrypt,
            sops_file_path,
            repository
        }
    }
}

/// Get the decrypted Kubernetes object from the RPC server
/// 
/// # Arguments
/// * `spec` - &DecryptorSpec
pub async fn get_decrypted_kubernetes_object(spec: &DecryptorSpec) -> Result<String, Error> {
    info!("Rpc call to retrieve the decrypted kubernetes file...");
    let mut client = CrdServiceClient::connect("http://[::1]:50208").await?;

    // create the payload
    let payload = Payload::new(spec);

    // create a request and call the rpc server
    let req = Request::new(payload);
    let res = client.render(req).await
        .map_err(|err| Error::Rpc(err.to_string()))?;

    let tmpl = res.get_ref().resource.clone();
    
    info!("✅ Template has been rendered.");

    Ok(tmpl)
}