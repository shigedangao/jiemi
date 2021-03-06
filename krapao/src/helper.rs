use std::{fs, env};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::io::ErrorKind;
use super::err::Error;

// Constant
const SSH_KEYPATH: &str = "../../id_rsa";
const SSH_GIT_ENV: &str = "GIT_SSH_COMMAND";

/// Create / Overwrite the SSH key
/// 
/// # Arguments
/// * `key` - &str
pub fn set_ssh_key(key: &str) -> Result<(), Error> {
    fs::write(SSH_KEYPATH, key)?;

    let mut perm = fs::metadata(SSH_KEYPATH)?.permissions();
    perm.set_mode(0o600);
    fs::set_permissions(SSH_KEYPATH, perm)?;

    Ok(())
}

/// Export the ssh key to the environment variable of the os
pub fn export_ssh_key_to_env() {
    let arg = format!("ssh -i {} -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no", SSH_KEYPATH);
    env::set_var(SSH_GIT_ENV, arg);
}

/// Create path if the path does not exist
/// 
/// # Arguments
/// * `path` - &Path
pub fn create_path(path: &Path) -> Result<(), Error> {
    if let Err(err) = fs::create_dir_all(path) {
        if err.kind() == ErrorKind::AlreadyExists {
            info!("🔍 Path already exist");
            return Ok(())
        }

        return Err(Error::from(err))
    }

    info!("📜 Credentials path has been created");
    Ok(())
}