use std::env;
use config::{Config, File};
use serde::Deserialize;
use crate::err::Error;

// constant
const GIT_USERNAME: &str = "GIT_USERNAME";
const GIT_TOKEN: &str = "GIT_TOKEN";
const GIT_REPOSITORY: &str = "GIT_REPOSITORY";
const GIT_REPO_TARGET: &str = "GIT_CLONE_TARGET";
const GIT_SSH_KEY: &str = "GIT_SSH_KEY";
const SYNC_INTERVAL: &str = "SYNC_INTERVAL";

#[derive(Debug, Default, Deserialize)]
pub struct Env {
    #[serde(rename(deserialize = "GIT_USERNAME"))]
    pub username: Option<String>,
    #[serde(rename(deserialize = "GIT_TOKEN"))]
    pub token: Option<String>,
    #[serde(rename(deserialize = "GIT_REPOSITORY"))]
    pub repository: String,
    #[serde(rename(deserialize = "GIT_CLONE_TARGET"))]
    pub target: String,
    #[serde(rename(deserialize = "GIT_SSH_KEY"))]
    pub ssh: Option<String>,
    #[serde(rename(deserialize = "SYNC_INTERVAL"))]
    pub sync_interval: Option<u64>
}

impl Env {
    /// Create a new env handler
    fn new() -> Self {
        Env::default()
    }

    /// Retrieve the environment variable from two sources
    ///     - If exists. then use the file Env.toml
    ///     - Use the OS environment variable
    /// 
    /// # Arguments
    /// * `self` - Env
    fn set_env(self) -> Result<Self, Error> {
        match self.load_local_env() {
            Ok(res) => Ok(res),
            Err(_) => self.load_os_env()
        }
    } 

    /// Load environment variable from the Env.toml file
    /// 
    /// # Arguments
    /// * `&self` - &Env
    fn load_local_env(&self) -> Result<Env, Error> {
        info!("Loading local environment variable");
        let mut settings = Config::default();
        settings.merge(File::with_name("Env"))?;

        let env = settings.try_into::<Env>()?;
        
        Ok(env)
    }

    /// Load environment variable from the OS
    /// 
    /// # Arguments
    /// * `&self` - &Env
    fn load_os_env(&self) -> Result<Env, Error> {
        info!("Loading OS environment variable");
        let sync_interval = match env::var(SYNC_INTERVAL).ok() {
            Some(s) => s.parse::<u64>().ok(),
            None => None
        };

        Ok(Env {
            username: env::var(GIT_USERNAME).ok(),
            token: env::var(GIT_TOKEN).ok(),
            repository: env::var(GIT_REPOSITORY)?,
            target: env::var(GIT_REPO_TARGET)?,
            ssh: env::var(GIT_SSH_KEY).ok(),
            sync_interval
        })
    }
}

/// Load environment variables
pub fn load_env() -> Result<Env, Error> {
    info!("Loading environment variable...");
    Env::new().set_env()
}