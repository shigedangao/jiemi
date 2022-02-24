use k8s_openapi::ByteString;
use crate::err::Error;

/// Decode a Base64 to a string
///
/// # Arguments
/// * `base` - &ByteString
pub fn decode_byte(base: &ByteString) -> Result<String, Error> {
    match base64::decode(base.0.clone()) {
        Ok(res) => String::from_utf8(res)
            .map_err(|err| Error::DecodedBytes(err.to_string())),
        Err(_) => Ok(String::from_utf8_lossy(&base.0).to_string())
    }
}
