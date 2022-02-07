use config::{Config, File};
use serde::Deserialize;
use rand::Rng;
use crate::err::Error;
use crate::server::service::repo::proto::Payload;

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

    /// Load environment variable from the Env.toml file
    /// This is only used in the dev environment
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
}

impl From<Payload> for Env {
    fn from(p: Payload) -> Self {
        let mut rng = rand::thread_rng();
        let mut env = Env {
            repository: p.url,
            target: rng.gen::<u32>().to_string(),
            sync_interval: Some(1800),
            ..Default::default()
        };

        // compute the username
        if let Some(creds) = p.cred {
            env.username = creds.username;
            env.token = creds.token;
            env.ssh = creds.ssh;
        }

        env
    }
}

/// Load environment variables
pub fn load_env() -> Result<Env, Error> {
    info!("Loading environment variable...");
    Env::new().load_local_env()
}