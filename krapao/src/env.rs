use std::env;
use config::{Config, File};
use serde::Deserialize;
use crate::err::Error;

// constant
const GIT_USERNAME: &str = "GIT_USERNAME";
const GIT_TOKEN: &str = "GIT_TOKEN";
const GIT_REPOSITORY: &str = "GIT_REPOSITORY";
const GIT_REPO_TARGET: &str = "GIT_CLONE_TARGET";

#[derive(Debug, Default, Deserialize)]
pub struct Env {
    #[serde(rename(deserialize = "GIT_USERNAME"))]
    pub username: Option<String>,
    #[serde(rename(deserialize = "GIT_TOKEN"))]
    pub token: Option<String>,
    #[serde(rename(deserialize = "GIT_REPOSITORY"))]
    pub repository: String,
    #[serde(rename(deserialize = "GIT_CLONE_TARGET"))]
    pub target: String
}

impl Env {
    fn new() -> Self {
        Env::default()
    }

    fn set_env(mut self) -> Result<Self, Error> {
        match self.load_local_env() {
            Ok(res) => Ok(res),
            Err(_) => self.load_os_env()
        }
    } 

    fn load_local_env(&self) -> Result<Env, Error> {
        info!("Loading local environment variable");
        let mut settings = Config::default();
        settings.merge(File::with_name("Env"))?;

        let env = settings.try_into::<Env>()?;
        
        Ok(env)
    }

    fn load_os_env(&self) -> Result<Env, Error> {
        info!("Loading OS environment variable");
        Ok(Env {
            username: Some(env::var(GIT_USERNAME)?),
            token: Some(env::var(GIT_TOKEN)?),
            repository: env::var(GIT_REPOSITORY)?,
            target: env::var(GIT_REPO_TARGET)?
        })
    }
}

pub fn load_env() -> Result<Env, Error> {
    info!("Loading environment variable...");
    Env::new().set_env()
}