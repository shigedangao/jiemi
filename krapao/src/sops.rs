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
    let file_path = config.target.to_str().unwrap_or_default();
    let t_file_path = format!("{}/{target_file_path}", file_path);
    let s_file_path = format!("{}/{sops_file_path}", file_path);
    
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
        let stderr = String::from_utf8(cmd.stderr)
            .map_err(|err| Error::Sops(err.to_string()))?;
        
        Err(Error::Sops(stderr))
    }
}