use std::process::Command;
use std::fs;
use crate::err::Error;

// Constant
const KEY_FILE_PATH: &str = "../private.rsa";
const GPG_AUTH_ERR: &str = "Unable to verify the imported private key";

/// Authenticate with pgp by creating the private.rsa key and then 
/// importing the key with the gpg command
/// 
/// # Arguments
/// * `key` - &str
pub fn authenticate_with_pgp(key: &str) -> Result<(), Error> {
    // write the private.rsa file
    fs::write(KEY_FILE_PATH, key)?;

    let status = Command::new("gpg")
        .arg("--import")
        .arg(KEY_FILE_PATH)
        .status()?;

    if !status.success() {
        return Err(Error::Sops(GPG_AUTH_ERR.to_owned()));
    }

    info!("🔑 PGP key registered");

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn expect_to_not_register_private_key() {
        let dummy_private_key = "foo-bar";
        let res = super::authenticate_with_pgp(dummy_private_key);

        assert!(res.is_err());
    }
}