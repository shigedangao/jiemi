use kube::{
    Api,
    Client,
    api::ListParams
};
use kube::runtime::{
    watcher,
    watcher::Event
};
use gen::crd::{
    Decryptor,
    status::{SyncStatus, DecryptorStatus}
};
use futures::{TryStreamExt, StreamExt};
use crate::err::Error;
use crate::state;
use crate::client::{server, crd};

pub mod apply;

/// Parse the decryptor struct which we're going to use to add the Status structure
/// 
/// # Arguments
/// * `object` - Decryptor
/// * `client` - Client
/// * `state` - State 
async fn parse_update_of_crd(object: Decryptor, client: Client, state: state::State) -> Result<(), Error> {
    let (name, generation_id, ns) = object.get_metadata_info()?;
    info!("ℹ️ Change has been detected on {name}");

    // If the resource is not registered in the state, then this mean that the repository
    // might not be pulled. In that case we call the rpc server to pull the repository
    if !state::is_registered(state.clone(), &name)? {
        // proceed to call the grpc api to pull the repo
        server::dispatch_clone_repository(&object.spec, &client, &ns).await?;
    }

    // In order to not create an infinite loop of update...
    // we're checking the generation_id
    let generation_exist = state::upsert_state(state, &name, generation_id)?;
    if generation_exist {
        info!("no need to update the status for decryptor {name}");
        return Ok(())
    }

    // Call the rpc server to get the decrypted k8s file to apply
    let (tmpl, hash) = match crd::get_decrypted_kubernetes_object(&object.spec, &ns).await {
        Ok(res) => res,
        Err(err) => {
            // Update the status of the current decryptor
            DecryptorStatus::new(
                SyncStatus::Error, 
                Some(err.to_string()), 
                None, 
                &object
            ).update_status(&name, &ns).await?;

            return Ok(())
        }
    };

    let apply_res = apply::apply_rendered_object(tmpl, &client, &ns).await;
    // if an error happened while applying the rendered object. Then set an error to the crd
    if let Err(err) = apply_res {
        DecryptorStatus::new(
            SyncStatus::Unsync, 
            Some(err.to_string()), 
            Some(hash), 
            &object
        ).update_status(&name, &ns).await?;

        return Ok(())
    }

    // Otherwise update has been successsful so add a sync status
    DecryptorStatus::new(
        SyncStatus::Sync, 
        None, 
        Some(hash), 
        &object
    )
        .update_status(&name, &ns).await?;

    Ok(())
}

/// Process the delete CRD
/// 
/// # Arguments
/// * `crd` - Decryptor
/// * `state` - State
fn deleted_crd(crd: Decryptor, state: state::State) -> Result<(), Error> {
    let (name, _, _) = crd.get_metadata_info()?;
    state::delete_item_in_state(state, &name)?;
    info!("🗑️ {name} has been removed");

    Ok(())
}

/// Create a watcher which will watch the Decryptor resources.
/// For each Decryptor resource that has been:
///     - created
///     - updated
/// 
/// The watcher will update the resource and add a status about the Decryptor
/// 
/// # Why use a State ?
/// By default, any changes on the Kubernetes object will trigger a new event.
/// This is something we want to avoid as this would create an infinite loop.
/// By using a state we're storing the generation id of the resource in a HashMap
/// Hence, when a new event happened. We'll check first the generation to see if it's the same
/// or not. If the id is similar we'll skip the update of the status
/// 
/// # Why watching the Decryptor resource ?
/// Using a watcher enabled to process the Decryptor object. This allows us to then
///     - Retrieve the Decryptor struct
///     - Clone the repository specified
///     - Decrypt the file specified in the spec
///     - Apply the file in the Cluster 
/// 
/// # Arguments
/// * `state` - State
pub async fn boostrap_watcher(state: state::State) -> Result<(), Error> {
    info!("Starting up the controller...");
    info!("Initializing client");
    let client = Client::try_default().await?;
    
    // Watch the Decryptor ressources
    let api: Api<Decryptor> = Api::all(client.clone());
    let mut watcher = watcher(api, ListParams::default()).boxed();

    // Event to listen for create / modified event on the Decryptor resources
    // let mut apply_events = try_flatten_applied(watcher).boxed_local();
    while let Some(event) = watcher.try_next().await? {
        let state = state.clone();
        let client = client.clone();

        match event {
            Event::Applied(dec) => {
                // spawn in a separate thread in order to process the update asynchronously
                tokio::spawn( async move {
                    let res = parse_update_of_crd( dec, client, state).await;
                    if let Err(err) = res {
                        error!("{err}");
                    }
                });
            },
            Event::Deleted(dec) => {
                if let Err(err) = deleted_crd(dec, state) {
                    error!("{err}")
                }
            },
            _ => {}
        };
    }
    
    Ok(())
}