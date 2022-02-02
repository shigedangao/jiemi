#[derive(Debug)]
pub enum Error {
    MissingMetadata(String)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingMetadata(key) => write!(f, "Key: {key} is not present within the metadata")
        }
    }
}

impl std::error::Error for Error {}