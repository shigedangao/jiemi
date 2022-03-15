use crate::err::Error;
use crate::server::service::crd::proto::Payload;

pub mod gcp;
pub mod aws;
pub mod pgp;

// Constant
const MISSING_PROVIDER_ERR: &str = "Missing provider";

#[derive(Debug)]
pub enum Provider {
    Gcp(String),
    Aws(String, String, String),
    Pgp(String),
    Vault,
    None
}

impl Provider {
    /// Create a new Provider from the Payload
    /// 
    /// # Arguments
    /// * `payload` - &Payload
    pub fn new(payload: &Payload) -> Self {
        if let Some(gcp) = payload.gcp.clone() {
            return Provider::Gcp(gcp.credentials);
        }

        if let Some(aws) = payload.aws.clone() {
            return Provider::Aws(
                aws.aws_access_key_id,
                aws.aws_secret_access_key,
                aws.region
            )
        }

        if let Some(key) = payload.pgp.clone() {
            return Provider::Pgp(key.private_key)
        }

        if payload.vault.is_some() {
            return Provider::Vault
        }

        Provider::None
    }

    /// Authenticate with the provider
    /// 
    /// # Arguments
    /// * `&self` - &Self
    pub fn authenticate(&self) -> Result<(), Error> {
        match self {
            Provider::Gcp(credentials) => gcp::set_authentication_file_for_gcp(credentials),
            Provider::Aws(id, key, region) => aws::authenticate(id, key, region),
            Provider::Pgp(key) => pgp::authenticate_with_pgp(key),
            Provider::Vault => Ok(()),
            Provider::None => Err(Error::ProviderAuth(MISSING_PROVIDER_ERR.to_owned()))
        }
    }
}
