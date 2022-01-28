use std::path::Path;
use std::process::{Command, ExitStatus};
use crate::err::Error;

// constant
const REPO_FOLDERNAME: &str = "../repo_clone";

#[derive(Debug)]
enum Credentials {
    Username(String, String),
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
struct GitConfig {
    auth_method: Credentials,
    repo_uri: String
}

impl GitConfig {
    fn new(auth_method: Credentials, repo_uri: &str) -> Result<Self, Error> {
        if repo_uri.is_empty() {
            return Err(Error::EmptyRepoURI);
        }
        
        Ok(GitConfig {
            auth_method,
            repo_uri: repo_uri.into()
        })
    }

    fn repo_exist(&self) -> Result<(), Error> {
       if Path::new(REPO_FOLDERNAME).is_dir() {
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
            Credentials::Username(username, password) => format!("{username}:{password}@{}", self.repo_uri),
            Credentials::Empty | Credentials::Ssh(_) => format!("{}", self.repo_uri)
        };

        // clone repository
        let status = Command::new("git").arg("clone").arg(uri).arg(REPO_FOLDERNAME).status()?;
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
        let handle = GitConfig::new(credentials, "https://github.com/shigedangao/gogo.git").unwrap();
        
        assert!(handle.repo_exist().is_ok());
    }

    #[test]
    fn expect_to_not_clone_repo() {
        let credentials = Credentials::Empty;
        let handle = GitConfig::new(credentials, "");
        assert_eq!(handle.unwrap_err(), Error::EmptyRepoURI);
    }
}