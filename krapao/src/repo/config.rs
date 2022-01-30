use std::path::Path;
use std::process::Command;
use crate::err::Error;
use crate::helper;
use crate::env::Env;

#[derive(Debug)]
pub enum Credentials {
    Token(String, String),
    // @TODO implement ssh, hopefully without braking mine local
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
    /// * `env` - &Env
    pub fn new(env: &Env) -> Self {
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

#[derive(Default, Debug)]
pub struct GitConfig {
    auth_method: Credentials,
    repo_uri: String,
    target: String
}

impl GitConfig {
    /// Create a new GitConfig handler
    /// 
    /// # Arguments
    /// * `auth_method` - Credentials
    /// * `repo_uri` - &str
    /// * `target` - &str
    pub fn new(auth_method: Credentials, repo_uri: &str, target: &str) -> Result<Self, Error> {
        if repo_uri.is_empty() {
            return Err(Error::EmptyRepoURI);
        }
        
        Ok(GitConfig {
            auth_method,
            repo_uri: repo_uri.to_owned(),
            target: target.to_owned()
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

    fn build_clone_uri(&self) -> Result<String, Error> {
        match &self.auth_method {
            Credentials::Token(username, token) => {
                let git_uri: Vec<&str> = self.repo_uri.split("https://").collect();
                match git_uri.get(1) {
                    Some(uri) => Ok(format!("https://{username}:{token}@{}", uri)),
                    None => return Err(Error::MalformattedURI)
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
        match &self.auth_method {
            Credentials::Ssh(key) => {
                helper::set_ssh_key(key)?;
                helper::export_ssh_key_to_env();
            },
            _ => {}
        };

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env;

    #[test]
    fn expect_to_clone_public_repo() {
        let credentials = Credentials::Empty;
        let handle = GitConfig::new(
            credentials, 
            "https://github.com/shigedangao/gogo.git",
             "../../test"
        ).unwrap();
        
        assert!(handle.init_repository().is_ok());
    }

    #[test]
    fn expect_to_not_clone_repo() {
        let credentials = Credentials::Empty;
        let handle = GitConfig::new(credentials, "", "");
        assert_eq!(handle.unwrap_err(), Error::EmptyRepoURI);
    }

    #[test]
    fn expect_to_clone_private_repo() {
        // read the env as the token is stored in the env
        let env = env::load_env().unwrap();
        let credentials = Credentials::Token(env.username.unwrap(), env.token.unwrap());
        let handle = GitConfig::new(
            credentials, 
            "https://github.com/shigedangao/mask-kube.git",
            "../../gogo"
        ).unwrap();

        assert!(handle.init_repository().is_ok());
    }

    #[test]
    fn expect_to_clone_repo_by_ssh() {
        let env = env::load_env().unwrap();
        let credentials = Credentials::Ssh(env.ssh.unwrap());
        let handle = GitConfig::new(
            credentials,
            "git@github.com:shigedangao/wurkflow.git",
            "../../wurkflow"
        ).unwrap();

        assert!(handle.init_repository().is_ok());
    }
}