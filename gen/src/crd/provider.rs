use kube::Client;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use super::repo::GenericConfig;
use crate::err::Error;

pub enum ProviderList {
    Gcp(String),
    Aws(String, String, String),
    Pgp(String),
    None
}

#[derive(Debug, JsonSchema, Serialize, Deserialize, Clone)]
pub struct GcpCredentials {
    service_account: GenericConfig
}

#[derive(Debug, JsonSchema, Serialize, Deserialize, Clone)]
pub struct AwsCredentials {
    key_id: GenericConfig,
    access_key: GenericConfig,
    region: GenericConfig
}

#[derive(Debug, JsonSchema, Serialize, Deserialize, Clone)]
pub struct PgpCredentials {
    #[serde(rename = "privateKey")]
    private_key: GenericConfig
}

#[async_trait]
pub trait AsyncTryFrom {
    type Error;
    type Output;

    /// Convert a Credential to a ProviderList. Because async method can't be used on trait. We have to implement a From
    /// method with async_trait crate
    /// 
    /// # Arguments
    /// * `&self` - &Self
    /// * `client` - Client
    /// * `ns` - &str
    async fn convert(&self, client: Client, ns: &str) -> Result<Self::Output, Self::Error>;
}

#[async_trait]
impl AsyncTryFrom for GcpCredentials {
    type Error = Error;
    type Output = ProviderList;

    async fn convert(&self, client: Client, ns: &str) -> Result<Self::Output, Self::Error> {
        let value = self.service_account.get_value(&client, ns).await?;

        Ok(ProviderList::Gcp(value))
    }
}

#[async_trait]
impl AsyncTryFrom for AwsCredentials {
    type Error = Error;
    type Output = ProviderList;

    async fn convert(&self, client: Client, ns: &str) -> Result<Self::Output, Self::Error> {
        let id = self.key_id.get_value(&client, ns).await?;
        let key = self.access_key.get_value(&client, ns).await?;
        let region = self.region.get_value(&client, ns).await?;

        Ok(ProviderList::Aws(id, key, region))
    }
}

#[async_trait]
impl AsyncTryFrom for PgpCredentials {
    type Error = Error;
    type Output = ProviderList;

    async fn convert(&self, client: Client, ns: &str) -> Result<Self::Output, Self::Error> {
        let key = self.private_key.get_value(&client, ns).await?;

        Ok(ProviderList::Pgp(key))
    }
}