use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use kube::{
    CustomResource,
    CustomResourceExt
};

#[derive(Debug, CustomResource, Serialize, Deserialize, Clone, JsonSchema)]
#[kube(group = "jiemi.cr", version = "v1alpha1", kind = "Decryptor", namespaced)]
pub struct DecryptorSpec {
    provider: Provider,
    source: Source
}

#[derive(Debug, JsonSchema, Clone, Serialize, Deserialize)]
struct Provider {
    gcp: Option<GenericConfig>,
    aws: Option<GenericConfig>
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
struct GenericConfig {
    secret_name: Option<String>,
    key: Option<String>,
    literal: Option<String>
}

#[derive(Debug, JsonSchema, Clone, Serialize, Deserialize)]
struct Source {
    repository: Repository,
    filename: String
}

#[derive(Debug, JsonSchema, Clone, Serialize, Deserialize)]
struct Repository {
    url: String,
    credentials: RepositoryCredentials
}

#[derive(Debug, JsonSchema, Clone, Serialize, Deserialize)]
struct RepositoryCredentials {
    username: Option<GenericConfig>,
    token: Option<GenericConfig>,
    ssh: Option<GenericConfig>
}

/// Generate a CRD which is used to be applied in a Kubernetes cluster
///     The final example of how the crd looks can be founded on the example folder
pub fn generate_crd() -> Result<String, Box<dyn std::error::Error>> {
    let res = serde_yaml::to_string(&Decryptor::crd())?;

    Ok(res)
}