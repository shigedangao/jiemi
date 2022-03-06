use kube::Client;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use crate::err::Error;
use super::secret::GenericConfig;

// Constant
const TOKEN_MSG_ERR: &str = "Unable to retrieve username / token for secret";
const SSH_MSG_ERR: &str = "SSH could not be founded";

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

        Err(Error::Kube(TOKEN_MSG_ERR.to_owned()))
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

        Err(Error::Kube(SSH_MSG_ERR.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_credentials() -> RepositoryCredentials {
        RepositoryCredentials {
            username: Some(GenericConfig {
                literal: Some("username".to_owned()),
                ..Default::default()
            }),
            token: Some(GenericConfig {
                literal: Some("token".to_owned()),
                ..Default::default()
            }),
            ssh: Some(GenericConfig {
                literal: Some("ssh".to_owned()),
                ..Default::default()
            })
        }
    }

    #[tokio::test]
    async fn expect_to_get_token_creds() {
        let credentials = get_credentials();
        let client = Client::try_default().await.unwrap();

        let (username, token) = credentials.get_token_creds(&client, "default").await.unwrap();

        assert_eq!(username, "username");
        assert_eq!(token, "token");
    }

    #[tokio::test]
    async fn expect_to_get_ssh_creds() {
        let credentials = get_credentials();
        let client = Client::try_default().await.unwrap();

        let ssh = credentials.get_ssh(&client, "default").await.unwrap();

        assert_eq!(ssh, "ssh");
    }

    #[tokio::test]
    async fn expect_to_not_retrieve_token() {
        let mut credentials = get_credentials();
        credentials.token.take();

        let client = Client::try_default().await.unwrap();
        let res = credentials.get_token_creds(&client, "default").await;

        assert!(res.is_err())
    }

    #[tokio::test]
    async fn expect_to_not_get_ssh() {
        let mut credentials = get_credentials();
        credentials.ssh.take();

        let client = Client::try_default().await.unwrap();
        let ssh = credentials.get_ssh(&client, "default").await;

        assert!(ssh.is_err());
    }
}