use crate::err::Error;
use crate::env::Env;
use self::config::{Credentials, GitConfig};

mod config;
pub mod fetch;

/// Initialize the git repository handler
/// 
/// # Arguments
/// * `env` - &Env
pub fn initialize_git(env: &Env) -> Result<GitConfig, Error> {
    // retrieve the environment variable for git credentials
    let credentials = Credentials::new(env);

    let config = GitConfig::new(credentials, &env.repository, &env.target)?;
    config.init_repository()?;

    Ok(config)
}
