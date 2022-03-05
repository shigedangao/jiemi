use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Serialize, Deserialize};
use crate::err::Error;
use crate::helper;
use crate::env::GitCredentials;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Credentials {
    Token(String, String),
    Ssh(String),
    Empty
}

impl Default for Credentials {
    fn default() -> Self {
        Credentials::Empty
    }
}

impl Credentials {
    /// Create a new Credential from the Env
    /// 
    /// # Arguments
    /// * `env` - &GitCredentials
    pub fn new(env: &GitCredentials) -> Self {
        let credentials = env.username.clone().zip(env.token.clone());
        if let Some((username, token)) = credentials {
            return Credentials::Token(username, token);
        }

        if let Some(key) = &env.ssh {
            return Credentials::Ssh(key.to_owned());
        }

        Credentials::Empty
    }
} 

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    auth_method: Credentials,
    pub repo_uri: String,
    pub target: PathBuf
}

impl GitConfig {
    /// Create a new GitConfig handler
    /// 
    /// # Arguments
    /// * `auth_method` - Credentials
    /// * `repo_uri` - &str
    /// * `target` - PathBuf
    pub fn new(auth_method: Credentials, repo_uri: &str, target: PathBuf) -> Result<Self, Error> {
        if repo_uri.is_empty() {
            return Err(Error::EmptyRepoURI);
        }
        
        Ok(GitConfig {
            auth_method,
            repo_uri: repo_uri.to_owned(),
            target
        })
    }

    /// Init the repository. Create if the repo exist or just skip it
    /// 
    /// # Arguments
    /// * `&self` - &GitConfig
    pub fn init_repository(&self) -> Result<(), Error> {
       if Path::new(&self.target).is_dir() {
           info!("Repository exist");
           return Ok(());
       }

       // clone the repo
       self.clone_repository()
    }

    /// Build the url which will be use by the git command
    /// to clone the target repository
    /// 
    /// # Arguments
    /// * `&self` - &Self
    fn build_clone_uri(&self) -> Result<String, Error> {
        match &self.auth_method {
            Credentials::Token(username, token) => {
                let git_uri: Vec<&str> = self.repo_uri.split("https://").collect();
                match git_uri.get(1) {
                    Some(uri) => Ok(format!("https://{username}:{token}@{}", uri)),
                    None => Err(Error::MalformattedURI)
                }
            },
            Credentials::Ssh(_) => {
                if !self.repo_uri.contains("git@") {
                    return Err(Error::MalformattedURI);
                }

                Ok(self.repo_uri.to_string())
            }
            Credentials::Empty => Ok(self.repo_uri.to_string())
        }
    }

    /// Building a git command to clone the repository
    /// 
    /// # Arguments
    /// * `&self` - &GitConfig
    fn clone_repository(&self) -> Result<(), Error> {
        info!("Cloning repository");
        // build git command args..
        let uri = self.build_clone_uri()?;

        // in the case of SSH we're writing the SSH key to a file
        // and we're setting an environment variable which should
        // allow us to clone the private repo
        if let Credentials::Ssh(key) = &self.auth_method {
            helper::set_ssh_key(key)?;
            helper::export_ssh_key_to_env();
        }

        // clone repository
        let status = Command::new("git")
            .arg("clone")
            .arg(uri)
            .arg(&self.target)
            .status()?;
        
        if !status.success() {
            return Err(Error::Clone(status.to_string()))
        }

        info!("Repository has been clone in the path {}", self.repo_uri);

        Ok(())
    }

    /// Delete repository that was clone
    pub fn delete_repository(&self) -> Result<(), Error> {
        info!("Delete repository {}", self.repo_uri);
        fs::remove_dir_all(&self.target)?;

        Ok(())
    } 

    /// Pull repository with the rebase option. Though we won't make any
    /// kind of changes to the original files
    /// 
    /// # Arguments
    /// * `&self` - &Self
    /// * `target` - String
    pub fn pull(&self) -> Result<(), Error> {
        info!("Pulling change from upstream for {}", self.repo_uri);
        let status = Command::new("git")
            .arg("-C")
            .arg(self.target.clone())
            .arg("pull")
            .arg("--rebase")
            .status()?;

        if !status.success() {
            error!("Fail to pull repository");
            return Err(Error::Pull(status.to_string()));
        }

        info!("Local repository cache has been updated");
        Ok(())
    }

    /// Get the commit hash from the repository
    /// 
    /// # Arguments
    /// * `&self` - &Self
    pub fn get_commit_hash(&self) -> Option<String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(self.target.clone())
            .arg("rev-parse")
            .arg("HEAD")
            .output();

        if let Ok(o) = output {
            let out = o.stdout;
            return String::from_utf8(out).ok();
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use config::{Config, File};
    use super::*;

    #[derive(Deserialize)]
    struct TestRepoConfig {
        #[serde(rename(deserialize = "GIT_USERNAME"))]
        username: String,
        #[serde(rename(deserialize = "GIT_TOKEN"))]
        token: String,
        #[serde(rename(deserialize = "GIT_SSH_KEY"))]
        ssh: String
    }

    /// Load environment variable from the Env.toml file
    /// This is only used in the dev environment. this method is only used for local test purposes
    /// 
    /// # Arguments
    /// * `&self` - &Env
    fn load_local_env() -> Result<TestRepoConfig, Error> {
        info!("Loading local environment variable");
        let mut settings = Config::default();
        settings.merge(File::with_name("Env"))?;

        let env = settings.try_into::<TestRepoConfig>()?;
        
        Ok(env)
    }

    #[test]
    fn expect_to_clone_public_repo() {
        let credentials = Credentials::Empty;
        let handle = GitConfig::new(
            credentials, 
            "https://github.com/shigedangao/gogo.git",
             PathBuf::from("../../test")
        ).unwrap();
        
        assert!(handle.init_repository().is_ok());

        // pull the repository
        let res = handle.pull();
        assert!(res.is_ok());
    }

    #[test]
    fn expect_to_not_clone_repo() {
        let credentials = Credentials::Empty;
        let handle = GitConfig::new(credentials, "", PathBuf::new());
        assert_eq!(handle.unwrap_err(), Error::EmptyRepoURI);
    }

    #[test]
    fn expect_to_clone_private_repo() {
        // read the env as the token is stored in the env
        let env = load_local_env().unwrap();
        let credentials = Credentials::Token(env.username, env.token);
        let handle = GitConfig::new(
            credentials, 
            "https://github.com/shigedangao/mask-kube.git",
            PathBuf::from("../../gogo")
        ).unwrap();

        assert!(handle.init_repository().is_ok());
    }

    #[test]
    fn expect_to_clone_repo_by_ssh() {
        let env = load_local_env().unwrap();
        let credentials = Credentials::Ssh(env.ssh);
        let handle = GitConfig::new(
            credentials,
            "git@github.com:shigedangao/wurkflow.git",
            PathBuf::from("../../wurkflow")
        ).unwrap();

        assert!(handle.init_repository().is_ok());
    }
}