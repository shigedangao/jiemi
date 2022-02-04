use core::fmt;
use kube::Error as KubeError;
use gen::err::Error as GenError;

#[derive(Debug)]
pub enum Error {
    KubeAuthentication,
    KubeRuntime,
    Generator(String),
    Watch(String),
    Serialize
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::KubeAuthentication => write!(f, "Unable to authenticate with Kubernetes"),
            Error::KubeRuntime => write!(f, "Unexpected error with the controller"),
            Error::Generator(msg) => write!(f, "Error encountered with the gen {msg}"),
            Error::Watch(msg) => write!(f, "Error while watching the decryptor resource {msg}"),
            Error::Serialize => write!(f, "Error while serializing the Status")
        }
    }
}

impl std::error::Error for Error {}

impl From<KubeError> for Error {
    fn from(err: KubeError) -> Self {
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

impl From<kube::runtime::watcher::Error> for Error {
    fn from(err: kube::runtime::watcher::Error) -> Self {
        Error::Watch(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Error::Serialize
    }
}