use std::process::Command;
use crate::repo::config::GitConfig;
use crate::err::Error;

/// Decrypt SOPS file and return the output to the caller
/// 
/// # Arguments
/// * `config` - &GitConfig
/// * `target_file_path` - &str
/// * `sops_file_path` - &str
pub fn decrypt_file(config: &GitConfig, target_file_path: &str, sops_file_path: &str) -> Result<String, Error> {
    let mut t_file_path = config.target.clone();
    t_file_path.push(target_file_path);

    let mut s_file_path = config.target.clone();
    s_file_path.push(sops_file_path);

    info!("Trying to decrypt {target_file_path}...");
    let cmd = Command::new("sops")
        .arg("-d")
        .arg(t_file_path)
        .arg("--config")
        .arg(s_file_path)
        .output()?;

    let status = cmd.status;
    if status.success() {
        let stdout = String::from_utf8(cmd.stdout)
            .map_err(|_| Error::Sops("Unable to parse SOPS stdout".to_owned()))?;
        
        Ok(stdout)
    } else {
        error!("Error while decrypting with SOPS {:?}", cmd.stderr);
        let stderr = String::from_utf8(cmd.stderr)
            .map_err(|err| Error::Sops(err.to_string()))?;
        
        Err(Error::Sops(stderr))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use super::*;
    use crate::auth::pgp;

    #[test]
    #[ignore]
    fn expect_to_decrypt_sops_file() {
        let encrypted_file_path = "../example/sops/encrypted.yaml";
        let sops_file_path = "../example/sops/.sops.yaml";

        let read = fs::read("../key/test_private_key.rsa").unwrap();
        let key = String::from_utf8(read).unwrap();
        pgp::authenticate_with_pgp(&key).unwrap();

        let config = GitConfig::default();
        let res = decrypt_file(&config, encrypted_file_path, sops_file_path);

        assert!(res.is_ok());
    }
}