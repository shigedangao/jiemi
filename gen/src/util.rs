use k8s_openapi::ByteString;
use crate::err::Error;

/// Decode a Base64 to a string
///
/// # Arguments
/// * `base` - &ByteString
pub fn decode_byte(base: &ByteString) -> Result<String, Error> {
    let decoded = base64::decode(base.0.clone())?;
    let str = String::from_utf8(decoded)
        .map_err(|err| Error::DecodedBytes(err.to_string()))?;

    Ok(str)
}