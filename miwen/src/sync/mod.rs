// This mod is used to pull changes from the repository
// from time to time and check whenever we need to update the resoruces
use kube::{Api, Client};
use kube::api::ListParams;
use gen::crd::status::{DecryptorStatus, SyncStatus};
use gen::crd::Decryptor;
use tokio::time::sleep;
use std::time::Duration;
use futures::future::try_join_all;
use crate::err::Error;
use crate::client::crd;
use crate::watcher::apply;

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
    let mut fut = Vec::new();
    for crd in crds {
        fut.push(get_and_apply_template(crd));
    }

    // Joining the futures better than spawning a thread for each crd
    let res = try_join_all(fut).await;
    if let Err(err) = res {
        return Err(err);
    }
    
    Ok(())
}

/// Get and apply the rendered template from the rpc server
/// 
/// # Arguments
/// * `mut decryptor` - Decryptor
async fn get_and_apply_template(mut decryptor: Decryptor) -> Result<(), Error> {
    let client = Client::try_default().await?;
    let (_, _, ns) = decryptor.get_metadata_info()?;
    // get the existing hash...
    let current_hash = match &decryptor.status {
        Some(st) => st.current.revision.clone(),
        None => String::new()
    };

    // get file and commit hash from the repo
    let spec = decryptor.spec.clone();
    let filename = &spec.source.file_to_decrypt;
    let (tmpl, hash) = crd::get_decrypted_kubernetes_object(&spec, &ns).await?;

    if current_hash != hash {
        // Apply the decrypted file in the kubernetes cluster
        info!("Found changes in repository. Apply changes for file {filename}");
        let apply_res = apply::apply_rendered_object(tmpl, &client, &ns).await;
        return match apply_res {
            Ok(_) => {
                decryptor.set_status(DecryptorStatus::new(
                    SyncStatus::Sync, 
                    None, 
                    Some(hash), 
                ));
                decryptor
                    .update_status()
                    .await
                    .map_err(Error::from)
            },
            Err(err) => {
                decryptor.set_status(DecryptorStatus::new(
                    SyncStatus::NotSync,  
                    Some(err.to_string()),  
                    Some(hash),  
                ));
                decryptor
                    .update_status()
                    .await
                    .map_err(Error::from)
            }
        }
    }

    info!("No change detected for {filename}");

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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn expect_list_crd_to_not_fail() {
        let client = Client::try_default().await.unwrap();
        let res = list_crd(client).await;

        assert!(res.is_ok());
    }
}