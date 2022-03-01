use std::process::Command;
use std::fs;
use dirs::home_dir;
use crate::err::Error;

// Constant
const KEY_FILE_PATH: &str = "keys/private.rsa";
const KEY_CREATE_ERR: &str = "Unable to store given private key for pgp operation";
const GPG_AUTH_ERR: &str = "Unable to verify the imported private key";

/// Authenticate with pgp by creating the private.rsa key and then 
/// importing the key with the gpg command
/// 
/// # Arguments
/// * `key` - &str
pub fn authenticate_with_pgp(key: &str) -> Result<(), Error> {
    // write the file in the folder
    let mut dir = home_dir()
        .ok_or_else(|| Error::Io(KEY_CREATE_ERR.to_owned()))?;
    dir.push(KEY_FILE_PATH);
    
    fs::write(&dir, key)?;

    let status = Command::new("gpg")
        .arg("--import")
        .arg(dir)
        .status()?;

    if !status.success() {
        return Err(Error::Sops(GPG_AUTH_ERR.to_owned()));
    }

    info!("🔑 PGP key registered");

    Ok(())
} 