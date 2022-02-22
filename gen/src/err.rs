#[derive(Debug)]
pub enum Error {
    MissingMetadata(String),
    Kube(String),
    DecodedBytes(String),
    Encoding(String)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingMetadata(key) => write!(f, "Key: {key} is not present within the metadata"),
            Error::Kube(msg) => write!(f, "Error while looking for kube resource {msg}"),
            Error::DecodedBytes(msg) => write!(f, "Unable to decoded bytes for reasons: {msg}"),
            Error::Encoding(msg) => write!(f, "Unable to encoded value to json: {msg}")
        }
    }
}

impl std::error::Error for Error {}

impl From<kube::Error> for Error {
    fn from(err: kube::Error) -> Self {
        Error::Kube(err.to_string())
    }
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Self {
        Error::DecodedBytes(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Encoding(err.to_string())
    }
}