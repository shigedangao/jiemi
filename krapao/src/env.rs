use serde::Deserialize;
use rand::Rng;
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