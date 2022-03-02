use crate::err::Error;
use crate::env::Env;
use self::config::{Credentials, GitConfig};

pub mod config;

/// Initialize the git repository handler
/// 
/// # Arguments
/// * `env` - &Env
pub fn initialize_git(env: &Env) -> Result<GitConfig, Error> {
    // retrieve the environment variable for git credentials
    let credentials = Credentials::new(env);

    let config = GitConfig::new(credentials, &env.repository, env.target.to_owned())?;
    config.init_repository()?;

    Ok(config)
}
