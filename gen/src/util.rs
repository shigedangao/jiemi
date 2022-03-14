use k8s_openapi::ByteString;
use crate::err::Error;

/// Decode a Base64 to a string
///
/// # Arguments
/// * `base` - &ByteString
pub(crate) fn decode_byte(base: &ByteString) -> Result<String, Error> {
    let value = String::from_utf8(base.0.clone())
        .map_err(|err| Error::DecodedBytes(err.to_string()))?;

    Ok(value)
}
