use kube::{Api, Client};
use kube::api::{ListParams, PostParams};
use kube::runtime::{
    watcher,
    utils::try_flatten_applied
};
use gen::crd::{
    Decryptor,
    status::{SyncStatus, DecryptorStatus}
};
use futures::{TryStreamExt, StreamExt};
use crate::err::Error;
use crate::state;
use crate::client::{server, crd};

mod apply;

/// Update the status of a targeted Decryptor object
/// 
/// # Arguments
/// * `client` - &Client
/// * `name` - &str
/// * `ns` - &str
/// * `status` - DecryptorStatus
async fn update_status(client: &Client, name: &str, ns: &str, status: DecryptorStatus) -> Result<(), Error> {
    let api = Api::<Decryptor>::namespaced(client.clone(), ns);
    let mut curr_decryptor_status = api.get_status(&name).await?;
    curr_decryptor_status.status = Some(status);

    api.replace_status(
        &name,
        &PostParams::default(),
        serde_json::to_vec(&curr_decryptor_status)?
    ).await?;

    Ok(())
}

/// Parse the decryptor struct which we're going to use to add the Status structure
/// 
/// # Arguments
/// * `object` - Decryptor
/// * `client` - Client
/// * `state` - State 
async fn parse_event(object: Decryptor, client: Client, state: state::State) -> Result<(), Error> {
    info!("Received event");
    let metadata = object.metadata.clone();
    // extract value which we'll use later
    let name = metadata.name
        .ok_or(Error::Watch("Name field does not exist on the Decryptor resource !".to_owned()))?;
    let generation_id = metadata.generation
        .ok_or(Error::Watch("Generation field does not exist in the Decryptor resource".to_owned()))?;
    let ns = metadata.namespace.unwrap_or("default".to_owned());

    // If the resource is not registered in the state, then this mean that the repository
    // might not be pulled. In that case we call the rpc server to pull the repository
    if !state::is_registered(state.clone(), &name)? {
        // proceed to call the grpc api to pull the repo
        server::dispatch_clone_repository(&object.spec, &client, &ns).await?;
    }

    // In order to not create an infinite loop of update...
    // we're checking the generation_id
    let generation_exist = state::gen_id_exist_from_state(state, name.clone(), generation_id)?;
    if generation_exist {
        info!("no need to update the status for decryptor {name}");
        return Ok(())
    }

    // Call the rpc server to get the decrypted k8s file to apply
    let (tmpl, hash) = match crd::get_decrypted_kubernetes_object(&object.spec).await {
        Ok(res) => res,
        Err(err) => {
            let status = DecryptorStatus::new(SyncStatus::Error, Some(err.to_string()), "".to_owned(), object);
            return update_status(&client, &name, &ns, status).await;
        }
    };

    let apply_res = apply::apply_rendered_object(tmpl, &client, &ns).await;
    // if an error happened while applying the rendered object. Then set an error to the crd
    if let Err(err) = apply_res {
        let status = DecryptorStatus::new(SyncStatus::Error, Some(err.to_string()), "".to_owned(), object);
        return update_status(&client, &name, &ns, status).await;
    }

    // Otherwise update has been successsful so add a sync status
    let status = DecryptorStatus::new(SyncStatus::Sync, None, hash, object);
    // update the status of the decryptor object
    update_status(&client, &name, &ns, status).await?;

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
    let watcher = watcher(api, ListParams::default());

    // Event to listen for create / modified event on the Decryptor resources
    let mut apply_events = try_flatten_applied(watcher).boxed_local();
    while let Some(dec) = apply_events.try_next().await? {
        // spawn in a separate thread in order to process the update asynchronously
        tokio::spawn( parse_event( dec, client.clone(), state.clone()));
    }
    
    Ok(())
}