use k8s_openapi::api::core::v1::Secret;
use kube::{Client, Api};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use crate::err::Error;
use crate::util;

#[derive(Debug, JsonSchema, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub url: String,
    pub credentials: Option<RepositoryCredentials>
}

#[derive(Debug, JsonSchema, Clone, Serialize, Deserialize)]
pub struct RepositoryCredentials {
    pub username: Option<GenericConfig>,
    pub token: Option<GenericConfig>,
    pub ssh: Option<GenericConfig>
}

impl RepositoryCredentials {
    /// Retrieve the GIT credentials for the field:
    ///     - username
    ///     - token
    /// 
    /// 
    /// # Arguments
    /// * `&self` - Self
    /// * `client` - &Client
    /// * `ns` - &str
    pub async fn get_token_creds(&self, client: &Client, ns: &str) -> Result<(String, String), Error> {
        let creds = self.username.as_ref().zip(self.token.as_ref());
        if let Some((u, t)) = creds {
            let username = match u.get_value(client, ns).await {
                Ok(u) => u,
                Err(err) => return Err(err)
            };
                
            let token = match t.get_value(client, ns).await {
                Ok(t) => t,
                Err(err) => return Err(err)
            };

            return Ok((username, token));
        }

        Err(Error::Kube("Unable to retrieve username / token for secret".to_owned()))
    }

    /// Get the SSH value from the CRD
    /// 
    /// # Arguments
    /// * `&self` - Self
    /// * `client` - &Client
    /// * `ns` - &str 
    pub async fn get_ssh(&self, client: &Client, ns: &str) -> Result<String, Error> {
        if let Some(conf) = self.ssh.as_ref() {
            let res = conf.get_value(client, ns).await?;
            return Ok(res);
        }

        Err(Error::Kube("SSH could not be founded".to_owned()))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct GenericConfig {
    secret_name: Option<String>,
    key: Option<String>,
    literal: Option<String>
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
    async fn get_value(&self, client: &Client, ns: &str) -> Result<String, Error> {
        if let Some(literal) = self.literal.to_owned() {
            return Ok(literal);
        }

        let creds = self.secret_name.as_ref().zip(self.key.as_ref());
        if let Some((secret_name, key)) = creds {
            let api: Api<Secret> = Api::namespaced(client.clone(), ns);
            let secret = api.get(&secret_name).await?;

            if let Some(data) = secret.data {
                if let Some(value) = data.get(key) {
                    let res = util::decode_byte(value)?;
                    return Ok(res);
                }
            }
        }

        Err(Error::Kube("Unable to retrieve the target value".to_owned()))
    }
}