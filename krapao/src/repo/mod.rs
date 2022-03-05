use crate::err::Error;
use crate::env::GitCredentials;
use self::config::{Credentials, GitConfig};

pub mod config;

/// Initialize the git repository handler
/// 
/// # Arguments
/// * `env` - &GitCredentials
pub fn initialize_git(env: &GitCredentials) -> Result<GitConfig, Error> {
    // retrieve the environment variable for git credentials
    let credentials = Credentials::new(env);

    let config = GitConfig::new(credentials, &env.repository, env.target.to_owned())?;
    config.init_repository()?;

    Ok(config)
}
