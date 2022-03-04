use std::time::Duration;
use gen::crd::{
    DecryptorSpec,
    provider::{ProviderList}
};
use tonic::Request;
use crate::err::Error;
use self::proto::{
    crd_service_client::CrdServiceClient,
    Payload,
    Gcp,
    Aws,
    Pgp
};
use super::REQUEST_TIMEOUT;

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
        let mut payload = Payload {
            file_to_decrypt,
            sops_file_path,
            repository,
            ..Default::default()
        };

        match credentials {
            ProviderList::Gcp(credentials) => {
                payload.gcp = Some(Gcp { credentials})
            },
            ProviderList::Aws(k, i, r) => {
                payload.aws = Some(Aws {
                    aws_access_key_id: k,
                    aws_secret_access_key: i,
                    region: r
                })
            },
            ProviderList::Pgp(k) => {
                payload.pgp = Some(Pgp {
                    private_key: k
                })
            }
            ProviderList::None => {}
        };

        Ok(payload)
    }
}

/// Get the decrypted Kubernetes object from the RPC server
/// 
/// # Arguments
/// * `spec` - &DecryptorSpec
/// * `ns` - &str
pub async fn get_decrypted_kubernetes_object(spec: &DecryptorSpec, ns: &str) -> Result<(String, String), Error> {
    info!("Rpc call to retrieve the decrypted kubernetes file...");
    let mut client = CrdServiceClient::connect(super::get_rpc_addr()).await?;

    // create the payload
    let payload = Payload::new(spec, ns).await?;

    // create a request and call the rpc server
    let mut req = Request::new(payload);
    req.set_timeout(Duration::from_secs(REQUEST_TIMEOUT));

    let res = client.render(req).await
        .map_err(|err| Error::Rpc(err.to_string()))?;

    let resp = res.get_ref();
    let tmpl = resp.resource.clone();
    let hash = resp.commit_hash.clone().unwrap_or_default();
    
    info!("✅ Template has been rendered.");

    Ok((tmpl, hash))
}