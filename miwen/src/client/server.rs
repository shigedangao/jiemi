use std::time::Duration;
use gen::crd::{DecryptorSpec, repo::RepositoryCredentials};
use kube::Client;
use tonic::Request;
use crate::err::Error;
use self::proto::{
    repo_service_client::RepoServiceClient,
    Payload,
    Credentials
};
use super::REQUEST_TIMEOUT;

mod proto {
    tonic::include_proto!("repository");
}

impl Credentials {
    /// Build the credentials needed to clone the repository
    /// 
    /// # Arguments
    /// * `rep` - RepositoryCredentials
    /// * `client` - &Client
    /// * `ns` - &str
    async fn build(rep: RepositoryCredentials, client: &Client, ns: &str) -> Result<Self, Error> {
        if rep.ssh.is_some() {
            let res = rep.get_ssh(client, ns).await?;
            return Ok(Credentials {
                ssh: Some(res),
                ..Default::default()
            });
        }

        let (username, token) = rep.get_token_creds(client, ns).await?;
        Ok(Credentials {
            username: Some(username),
            token: Some(token),
            ..Default::default()
        })
    }
}

/// Dispatch to krapao rpc server the repository to clone
///     - Build credentials needed for krapao to clone the repository if needed
/// 
/// # Arguments
/// * `spec` - &DecryptorSpec
/// * `kube_client` - &Client
/// * `ns` - &str
pub async fn dispatch_clone_repository(spec: &DecryptorSpec, kube_client: &Client, ns: &str) -> Result<(), Error> {
    info!("Rpc call to clone the target repository...");
    let mut client = RepoServiceClient::connect(super::get_rpc_addr()).await?;
    // request to grpc server
    // build credentials
    let spec_creds = spec.source.repository.credentials.clone();
    let cred = match spec_creds {
        Some(res) => Some(Credentials::build(res, kube_client, ns).await?),
        None => None
    };

    let mut req = Request::new(Payload {
        url: spec.source.repository.url.clone(),
        cred
    });
    req.set_timeout(Duration::from_secs(REQUEST_TIMEOUT));
    
    client.set_repository(req).await
        .map_err(|err| Error::Rpc(err.to_string()))?;
    
    info!("Repository has been setted up");

    Ok(())
}