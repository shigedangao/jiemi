use core::fmt;
use kube::Error as KubeError;
use gen::err::Error as GenError;

#[derive(Debug)]
pub enum Error {
    KubeAuthentication,
    KubeRuntime,
    Generator(String)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::KubeAuthentication => write!(f, "Unable to authenticate with Kubernetes"),
            Error::KubeRuntime => write!(f, "Unexpected error with the controller"),
            Error::Generator(msg) => write!(f, "Error encountered with the gen {msg}")
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

impl From<GenError> for Error {
    fn from(err: GenError) -> Self {
        Error::Generator(err.to_string())
    }
}