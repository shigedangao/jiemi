use std::ffi::OsString;
use tonic::Status;

#[derive(Debug, PartialEq)]
pub enum Error {
    Auth(String),
    EmptyRepoURI,
    MalformattedURI,
    Clone(String),
    Config(String),
    Pull(String),
    RefreshDuration,
    MaxPullRetry,
    Server(String),
    Bootstrap(String),
    Sync(String),
    Sops(String),
    Encoding(String)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Error::Auth(msg) => write!(f, "Unable to authenticate reasons: {msg}"),
            Error::EmptyRepoURI => write!(f, "Unable to clone repository. Url is empty"),
            Error::MalformattedURI => write!(f, "Repository URL is malformatted"),
            Error::Clone(msg) => write!(f, "Error while cloning repository {msg}"),
            Error::Config(msg) => write!(f, "Unable to parse the configuration spec to bootstrap service {msg}"),
            Error::Pull(msg) => write!(f, "Unable to pull repository changes {msg}"),
            Error::RefreshDuration => write!(f, "Refresh interval is inferior to 180 seconds / 3 min"),
            Error::MaxPullRetry => write!(f, "Failed to refresh repository after retrying 20 times"),
            Error::Server(msg) => write!(f, "gRPC server error: {msg}"),
            Error::Bootstrap(msg) => write!(f, "Initialization problem. State can't be recovered {msg}"),
            Error::Sync(msg) => write!(f, "Issue while syncing repositories {msg}"),
            Error::Sops(msg) => write!(f, "Error with SOPS: {msg}"),
            Error::Encoding(msg) => write!(f, "Error while encoding data: {msg}")
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Clone(err.to_string())
    }
}

impl From<OsString> for Error {
    fn from(_: OsString) -> Self {
        Error::Auth("Unable to parse the authentication credential".to_owned())
    }
}

impl From<std::env::VarError> for Error {
    fn from(err: std::env::VarError) -> Self {
        Error::Config(err.to_string())
    }
}

impl From<config::ConfigError> for Error {
    fn from(err: config::ConfigError) -> Self {
        Error::Config(err.to_string())
    }
}

impl From<tonic::transport::Error> for Error {
    fn from(err: tonic::transport::Error) -> Self {
        Error::Server(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Bootstrap(err.to_string())
    }
}

impl From<Error> for Status {
    fn from(err: Error) -> Self {
        Status::internal(err.to_string())
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::Sops(err.to_string())
    }
}