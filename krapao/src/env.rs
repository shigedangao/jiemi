use std::path::PathBuf;

use dirs::home_dir;
use serde::Deserialize;
use rand::Rng;
use crate::server::service::repo::proto::Payload;

// Constant
const REPOSITORY_PATH: &str = "workspace/repo";

#[derive(Debug, Default, Deserialize)]
pub struct Env {
    pub username: Option<String>,
    pub token: Option<String>,
    pub repository: String,
    pub target: PathBuf,
    pub ssh: Option<String>,
    pub sync_interval: Option<u64>
}

impl From<Payload> for Env {
    fn from(p: Payload) -> Self {
        let mut rng = rand::thread_rng();

        let mut dir = home_dir().unwrap_or_default();
        dir.push(REPOSITORY_PATH);
        dir.push(rng.gen::<u32>().to_string());

        let mut env = Env {
            repository: p.url,
            target: dir,
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