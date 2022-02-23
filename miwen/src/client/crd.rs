use gen::crd::{
    DecryptorSpec,
    ProviderList
};
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
    /// * `ns` - &str
    async fn new(spec: &DecryptorSpec, ns: &str) -> Result<Self, Error> {
        let repository = spec.source.repository.url.to_owned();
        let file_to_decrypt = spec.source.file_to_decrypt.to_owned();
        let sops_file_path = spec.source.sops_path.to_owned();
        
        // get the auth provider from the crd
        let provider = spec.provider.to_owned();
        let credentials = provider.get_credentials(ns).await?;

        let (provider_id, creds) = match credentials {
            ProviderList::Gcp(s) => (0, s),
            ProviderList::Aws(s) => (1, s),
            ProviderList::None => (2, "".to_owned())
        };

        Ok(Payload {
            file_to_decrypt,
            sops_file_path,
            repository,
            provider: provider_id,
            credentials: creds
        })
    }
}

/// Get the decrypted Kubernetes object from the RPC server
/// 
/// # Arguments
/// * `spec` - &DecryptorSpec
pub async fn get_decrypted_kubernetes_object(spec: &DecryptorSpec, ns: &str) -> Result<(String, String), Error> {
    info!("Rpc call to retrieve the decrypted kubernetes file...");
    let mut client = CrdServiceClient::connect("http://[::1]:50208").await?;

    // create the payload
    let payload = Payload::new(spec, ns).await?;

    // create a request and call the rpc server
    let req = Request::new(payload);
    let res = client.render(req).await
        .map_err(|err| Error::Rpc(err.to_string()))?;

    let resp = res.get_ref();
    let tmpl = resp.resource.clone();
    let hash = resp.commit_hash.clone().unwrap_or_default();
    
    info!("✅ Template has been rendered.");

    Ok((tmpl, hash))
}