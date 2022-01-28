#[derive(Debug, PartialEq)]
pub enum Error {
    Auth(String),
    EmptyRepoURI,
    Clone(String)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Error::Auth(msg) => write!(f, "Unable to authenticate to git {msg}"),
            Error::EmptyRepoURI => write!(f, "Unable to clone repository. Url is empty"),
            Error::Clone(msg) => write!(f, "Error while cloning repository {msg}")
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Clone(err.to_string())
    }
} 