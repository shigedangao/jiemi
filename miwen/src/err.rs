use core::fmt;
use kube::Error as KubeError;
use gen::err::Error as GenError;

#[derive(Debug)]
pub enum Error {
    KubeAuthentication,
    KubeRuntime,
    Generator(String),
    Watch(String),
    Serialize,
    Rpc(String),
    Apply(String)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::KubeAuthentication => write!(f, "Unable to authenticate with Kubernetes"),
            Error::KubeRuntime => write!(f, "Unexpected error with the controller"),
            Error::Generator(msg) => write!(f, "Error encountered with the gen {msg}"),
            Error::Watch(msg) => write!(f, "Error while watching the decryptor resource {msg}"),
            Error::Serialize => write!(f, "Error while serializing the Status"),
            Error::Rpc(msg) => write!(f, "Error while communicating with rpc server {msg}"),
            Error::Apply(msg) => write!(f, "Error while applying rendered resource from repo: {msg}")
            
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

impl From<tonic::transport::Error> for Error {
    fn from(err: tonic::transport::Error) -> Self {
        Error::Rpc(err.to_string())
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(_: serde_yaml::Error) -> Self {
        Error::Apply("Unable to decrypt the resource from the repository".to_owned())
    }
}