use crate::err::Error;
use crate::env::Env;
use self::config::{Credentials, GitConfig};

mod config;

pub fn initialize_git(env: &Env) -> Result<(), Error> {
    // retrieve the environment variable for git credentials
    let credentials = env.username.clone().zip(env.token.clone());
    let credentials = match credentials {
        Some((username, token)) => Credentials::Token(username, token),
        None => Credentials::Empty
    };

    GitConfig::new(credentials, &env.repository, &env.target)?
        .init_repository()?;

    Ok(())
}
