// This mod is used to pull changes from the repository
// from time to time and check whenever we need to update the resoruces
use kube::{Api, Client};
use kube::api::ListParams;
use gen::crd::status::{DecryptorStatus, SyncStatus};
use gen::crd::Decryptor;
use tokio::time::sleep;
use std::time::Duration;
use crate::err::Error;
use crate::client::crd;
use crate::watcher::{
    apply,
    DEFAULT_NAMESPACE,
};

// constant
const THREAD_SLEEP: u64 = 180;

/// Bootstrap the repo sync process
/// It runs every 180s / 3min
pub async fn bootstrap_repo_sync() -> Result<(), Error> {
    info!("Starting up sync process");
    loop {
        sleep(Duration::from_secs(THREAD_SLEEP)).await;

        tokio::spawn(async move {
            info!("Sync process is running...");
            if let Err(err) = sync_encrypted_file_with_git().await {
                error!("Error while syncing repository with cluster: {}", err.to_string());
            }
        });
    }
} 

/// Synchronize encrypted file specified in the CRD with the associated repository
/// Basically, we're comparing the commit hash of the last sync with the current hash in the
/// repository.
/// If the hash is different, then we may synchronize the file with the cluster
async fn sync_encrypted_file_with_git() -> Result<(), Error> {
    let client = Client::try_default().await?;
    let crds = list_crd(client.clone()).await?;

    // for each crd we're going to check whenever the crd is synced with the latest
    for crd in crds {
        let metadata = crd.metadata.clone();
        let name = metadata.name
            .ok_or(Error::Watch("Name field does not exist on the Decryptor resource !".to_owned()))?;
        let ns = metadata.namespace
            .unwrap_or(DEFAULT_NAMESPACE.to_owned());

        // get the existing hash...
        let current_hash = match &crd.status {
            Some(st) => st.current.revision.clone(),
            None => "".to_owned()
        };

        // get file and commit hash from the repo
        let spec = crd.spec.clone();
        let filename = &spec.source.file_to_decrypt;
        let (tmpl, hash) = crd::get_decrypted_kubernetes_object(&spec, &ns).await?;

        if current_hash != hash {
            // Apply the decrypted file in the kubernetes cluster
            info!("Found changes in repository. Apply changes for file {filename}");
            let apply_res = apply::apply_rendered_object(tmpl, &client, &ns).await;
            match apply_res {
                Ok(_) => {
                    let status = DecryptorStatus::new(SyncStatus::Sync, None, hash, crd);
                    status.update_status(&name, &ns).await?;
                },
                Err(err) => {
                    let status = DecryptorStatus::new(SyncStatus::Error,  Some(err.to_string()),  hash,  crd);
                    status.update_status(&name, &ns).await?;
                }
            };

            return Ok(())
        }

        info!("No change detected for {filename}");
    }


    Ok(())
}

/// Get a list of Crd. The list is used to get the file to apply on the cluster
/// We're assuming that the repo is being already pulled...
/// 
/// # Arguments
/// * `client` - Client
async fn list_crd(client: Client) -> Result<Vec<Decryptor>, Error> {
    let api: Api<Decryptor> = Api::all(client);
    let mut list = Vec::new();

    for crd in api.list(&ListParams::default()).await? {
        list.push(crd);
    }

    Ok(list)
}