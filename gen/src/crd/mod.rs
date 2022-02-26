use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use kube::{
    CustomResource,
    CustomResourceExt,
    Client
};
use status::DecryptorStatus;
use crate::err::Error;

pub mod status;
pub mod repo;

// Constant
const DEFAULT_NAMESPACE: &str = "default";

pub enum ProviderList {
    Gcp(String),
    Aws(String),
    None
}

// The implementation is based on
//
//  - https://github.com/kube-rs/kube-rs/blob/bf3b248f0c96b229863e0bff510fdf118efd2381/examples/crd_apply.rs
#[derive(Debug, CustomResource, Serialize, Deserialize, Clone, JsonSchema)]
#[kube(status = "DecryptorStatus")]
#[kube(group = "jiemi.cr", version = "v1alpha1", kind = "Decryptor", namespaced)]
pub struct DecryptorSpec {
    pub provider: Provider,
    pub source: Source
}

#[derive(Debug, JsonSchema, Clone, Serialize, Deserialize)]
pub struct Provider {
    gcp: Option<repo::GenericConfig>,
    aws: Option<repo::GenericConfig>
}

#[derive(Debug, JsonSchema, Clone, Serialize, Deserialize)]
pub struct Source {
    pub repository: repo::Repository,
    #[serde(rename = "fileToDecrypt")]
    pub file_to_decrypt: String,
    #[serde(rename = "sopsPath")]
    pub sops_path: String
}

/// Generate a CRD which is used to be applied in a Kubernetes cluster
///     The final example of how the crd looks can be founded on the example folder
pub fn generate_crd() -> Result<String, Box<dyn std::error::Error>> {
    let res = serde_yaml::to_string(&Decryptor::crd())?;

    Ok(res)
}

impl Provider {
    /// Get the credentials value from the provider section
    /// 
    /// # Arguments
    /// * `&self` - &Self
    /// * `ns` - &str
    pub async fn get_credentials(&self, ns: &str) -> Result<ProviderList, Error> {
        let client = Client::try_default().await?;
        if let Some(gcp) = self.gcp.clone() {
            let value = gcp.get_value(&client, ns).await?;
            return Ok(ProviderList::Gcp(value));
        }

        if let Some(aws) = self.aws.clone() {
            let value = aws.get_value(&client, ns).await?;
            return Ok(ProviderList::Aws(value));
        }

        Ok(ProviderList::None)
    }
}

impl Decryptor {
    /// Get the metadata info needed to perform some operation on the crd
    /// 
    /// # Arguments
    /// * `&self` - &Self
    pub fn get_metadata_info(&self) -> Result<(String, i64, String), Error> {
        let metadata = self.metadata.clone();

        let name = metadata.name
            .ok_or_else(|| Error::MissingMetadata("name".to_owned()))?;
        let generation_id = metadata.generation
            .ok_or_else(|| Error::MissingMetadata("generation_id".to_owned()))?;
        let ns = metadata.namespace
            .unwrap_or_else(|| DEFAULT_NAMESPACE.to_owned());

        Ok((name, generation_id, ns))
    }
}