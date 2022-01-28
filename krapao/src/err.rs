use std::{fmt::write, ffi::OsString};

#[derive(Debug, PartialEq)]
pub enum Error {
    Auth(String),
    EmptyRepoURI,
    MalformattedURI,
    Clone(String),
    Config(String)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Error::Auth(msg) => write!(f, "Unable to authenticate to git {msg}"),
            Error::EmptyRepoURI => write!(f, "Unable to clone repository. Url is empty"),
            Error::MalformattedURI => write!(f, "Repository URL is malformatted"),
            Error::Clone(msg) => write!(f, "Error while cloning repository {msg}"),
            Error::Config(msg) => write!(f, "Unable to parse the configuration spec to bootstrap service {msg}")
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