use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use kube::{Client, Api};
use k8s_openapi::api::core::v1::Secret;
use crate::util;
use crate::err::Error;

// Constant
const MISSING_SECRET_MSG_ERR: &str = "Secret has not been specified";

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, Default)]
pub struct GenericConfig {
    #[serde(rename = "secretName")]
    pub secret_name: Option<String>,
    pub key: Option<String>,
    pub literal: Option<String>
}

impl GenericConfig {
    /// Get the value from a secret or a literal value
    /// If a secret is specified. Then we're going to load the secret and get the value from the key
    /// If the valus is a literal then return it straight away
    /// 
    /// # Arguments
    /// * `&self` - Self
    /// * `client` - &Client
    /// * `ns` - &str
    pub async fn get_value(&self, client: &Client, ns: &str) -> Result<String, Error> {
        if let Some(literal) = self.literal.to_owned() {
            return Ok(literal);
        }

        let creds = self.secret_name.as_ref().zip(self.key.as_ref());
        if let Some((secret_name, key)) = creds {
            let api: Api<Secret> = Api::namespaced(client.clone(), ns);
            let secret = api.get(secret_name).await?;

            if let Some(data) = secret.data {
                if let Some(value) = data.get(key) {
                    let res = util::decode_byte(value)?;
                    return Ok(res);
                }
            }

            return Err(Error::Kube("Unable to decrypt the secret {&secret_name} with key {&key}".to_owned()));
        }

        Err(Error::Kube(MISSING_SECRET_MSG_ERR.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn expect_to_get_secret_value_from_secret_file() {
        let client = Client::try_default().await.unwrap();
        // required to have an existing kubernetes secret on the cluster for this test to pass...
        // use the file example/test/secret.yaml
        let config = GenericConfig {
            secret_name: Some("unit-test-secret".to_owned()),
            key: Some("foo".to_owned()),
            literal: None
        };

        let res = config.get_value(&client, "default").await.unwrap();
        assert_eq!(res, "bar");
    }

    #[tokio::test]
    async fn expect_to_get_literal_value() {
        let client = Client::try_default().await.unwrap();
        let config = GenericConfig {
            secret_name: None,
            key: None,
            literal: Some("foo".to_owned())
        };

        let res = config.get_value(&client, "default").await.unwrap();
        assert_eq!(res, "foo");
    }

    #[tokio::test]
    async fn expect_to_return_error() {
        let client = Client::try_default().await.unwrap();
        let config = GenericConfig::default();

        let res = config.get_value(&client, "default").await;
        assert!(res.is_err())
    }
}