use crate::err::Error;
use crate::server::service::crd::proto::payload::Provider as ProtoProvider;

pub mod gcp;

pub enum Provider {
    Gcp,
    Aws,
    None
}

impl Provider {
    /// Authenticate with the provider
    /// 
    /// # Arguments
    /// * `&self` - &Self
    /// * `credentials` - &str
    pub fn authenticate(&self, credentials: &str) -> Result<(), Error> {
        match self {
            Provider::Gcp => gcp::set_authentication_file_for_gcp(credentials),
            Provider::Aws => Ok(()),
            Provider::None => Ok(())
        }
    }
}

impl From<ProtoProvider> for Provider {
    fn from(p: ProtoProvider) -> Self {
        match p {
            ProtoProvider::Gcp => Provider::Gcp,
            ProtoProvider::Aws => Provider::Aws,
            ProtoProvider::None => Provider::None
        }
    }
}