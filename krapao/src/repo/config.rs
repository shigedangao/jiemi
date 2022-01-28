use std::path::Path;
use std::process::{Command};
use crate::err::Error;

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

#[derive(Default, Debug)]
pub struct GitConfig {
    auth_method: Credentials,
    repo_uri: String,
    target: String
}

impl GitConfig {
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

    pub fn init_repository(&self) -> Result<(), Error> {
       if Path::new(&self.target).is_dir() {
           info!("Repository exist");
           return Ok(());
       }

       // clone the repo
       self.clone_repository()
    }

    fn clone_repository(&self) -> Result<(), Error> {
        info!("Cloning repository");
        // build git command args..
        let uri = match &self.auth_method {
            Credentials::Token(username, token) => {
                let git_uri: Vec<&str> = self.repo_uri.split("https://").collect();
                match git_uri.get(1) {
                    Some(uri) => format!("https://{username}:{token}@{}", uri),
                    None => return Err(Error::MalformattedURI)
                }
            },
            Credentials::Empty | Credentials::Ssh(_) => format!("{}", self.repo_uri)
        };

        // clone repository
        let status = Command::new("git").arg("clone").arg(uri).arg(&self.target).status()?;
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
        let credentials = Credentials::Token("shigedangao".to_owned(), "REPLACE BY TOKEN".to_owned());
        let handle = GitConfig::new(
            credentials, 
            "https://github.com/shigedangao/mask-kube.git",
            "../../gogo"
        ).unwrap();

        assert!(handle.init_repository().is_ok());
    }
}