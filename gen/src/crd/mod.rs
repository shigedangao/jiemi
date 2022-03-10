use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use kube::{
    CustomResource,
    CustomResourceExt,
    Client,
    Api,
    api::PostParams
};
use status::DecryptorStatus;
use crate::err::Error;
use provider::AsyncTryFrom;

pub mod status;
pub mod repo;
pub mod provider;
pub mod secret;

// Constant
const DEFAULT_NAMESPACE: &str = "default";

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
    gcp: Option<provider::GcpCredentials>,
    aws: Option<provider::AwsCredentials>,
    pgp: Option<provider::PgpCredentials>
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
    pub async fn get_credentials(&self, ns: &str) -> Result<provider::ProviderList, Error> {
        let client = Client::try_default().await?;
        if let Some(gcp) = self.gcp.clone() {
            let list = gcp.convert(client, ns).await?;
            return Ok(list);
        }

        if let Some(aws) = self.aws.clone() {
            let list = aws.convert(client, ns).await?;
            return Ok(list);
        }

        if let Some(pgp) = self.pgp.clone() {
            let list = pgp.convert(client, ns).await?;
            return Ok(list);
        }

        Ok(provider::ProviderList::None)
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

    /// Set the status in the current Decryptor crd
    /// 
    /// # Arguments
    /// * `&mut self` - &Self
    /// * `status` - DecryptorStatus
    pub fn set_status(&mut self, status: DecryptorStatus) {
        self.status = Some(status);
    }

    /// Update the status of the Decrytpro
    /// 
    /// # Arguments
    /// * `&self` - &Self
    pub async fn update_status(&self) -> Result<(), Error> {
        let (name, _, ns) = self.get_metadata_info()?;
        let client = Client::try_default().await?;
        let api = Api::<Decryptor>::namespaced(client.clone(), &ns);
        
        api.replace_status(
            &name,
            &PostParams::default(),
            serde_json::to_vec(&self)?
        ).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use provider::{
        GcpCredentials,
        PgpCredentials,
        AwsCredentials,
        ProviderList
    };
    use secret::GenericConfig;

    #[tokio::test]
    async fn expect_to_get_gcp_credentials() {
        let provider = Provider {
            gcp: Some(GcpCredentials {
                service_account: GenericConfig {
                    literal: Some("google-credentials".to_owned()),
                    ..Default::default()
                }
            }),
            aws: None,
            pgp: None
        };

        let list = provider.get_credentials("default").await;
        assert!(list.is_ok());

        let list = list.unwrap();
        match list {
            ProviderList::Gcp(v) => assert_eq!(v, "google-credentials"),
            _ => panic!("Expect to return GCP credentials")
        }
    }

    #[tokio::test]
    async fn expect_to_get_pgp_credentials() {
        let provider = Provider {
            gcp: None,
            aws: None,
            pgp: Some(PgpCredentials {
                private_key: GenericConfig {
                    literal: Some("pgp-credentials".to_owned()),
                    ..Default::default()
                }
            })
        };

        let list = provider.get_credentials("default").await;
        assert!(list.is_ok());

        let list = list.unwrap();
        match list {
            ProviderList::Pgp(v) => assert_eq!(v, "pgp-credentials"),
            _ => panic!("Expect to return PGP credentials")
        }
    }

    #[tokio::test]
    async fn expect_to_get_aws_credentials() {
        let provider = Provider {
            gcp: None,
            aws: Some(AwsCredentials {
                key_id: GenericConfig {
                    literal: Some("key-id-credentials".to_owned()),
                    ..Default::default()
                },
                access_key: GenericConfig {
                    literal: Some("access-key-credentials".to_owned()),
                    ..Default::default()
                },
                region: GenericConfig {
                    literal: Some("region-credentials".to_owned()),
                    ..Default::default()
                }
            }),
            pgp: None
        };

        let list = provider.get_credentials("default").await;
        assert!(list.is_ok());

        let list = list.unwrap();
        match list {
            ProviderList::Aws(i, k, r) => {
                assert_eq!(i, "key-id-credentials");
                assert_eq!(k, "access-key-credentials");
                assert_eq!(r, "region-credentials");
            },
            _ => panic!("Expect to return AWS credentials")
        }
    }

    #[tokio::test]
    async fn expect_to_get_no_provider() {
        let provider = Provider {
            gcp: None,
            aws: None,
            pgp: None
        };

        let list = provider.get_credentials("default").await;
        assert!(list.is_ok());

        let list = list.unwrap();
        assert_eq!(list, ProviderList::None);
    }
}