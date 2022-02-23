use crate::err::Error;

pub mod gcp;

pub enum Provider {
    Gcp,
    Aws
}

impl Provider {
    /// Authenticate with the provider
    /// 
    /// # Arguments
    /// * `&self` - &Self
    /// * `credentials` - &str
    fn authenticate(&self, credentials: &str) -> Result<(), Error> {
        match self {
            Provider::Gcp => gcp::set_authentication_file_for_gcp(credentials),
            Provider::Aws => Ok(())
        }
    }
}