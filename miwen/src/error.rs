use core::fmt;
use kube::Error as KubeError;

#[derive(Debug)]
pub enum Error {
    KubeAuthentication,
    KubeRuntime
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::KubeAuthentication => write!(f, "Unable to authenticate with Kubernetes"),
            Error::KubeRuntime => write!(f, "Unexpected error with the controller")
        }
    }
}

impl std::error::Error for Error {}

impl From<KubeError> for Error {
    fn from(err: KubeError) -> Self {
        error!("{:?}", err);
        match err {
            KubeError::Auth(_) => Error::KubeAuthentication,
            _ => Error::KubeRuntime
        }
    }
} 