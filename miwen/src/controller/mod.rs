use std::sync::Arc;
use std::time::Duration;
use kube::{
    Client,
    runtime::{
        controller::{
            Controller, Context, ReconcilerAction
        },
        watcher,
        utils::try_flatten_applied
    },
    Api,
    api::{ListParams, PostParams},
    ResourceExt
};
use crate::error::Error;
use gen::crd::{
    Decryptor,
    status::{SyncStatus, DecryptorStatus}
};
use futures::{TryStreamExt, StreamExt};
use super::State;

async fn reconcile(generator: Decryptor, client: Client, state: State) -> Result<(), Error> {
    let mut inner_state = state.lock().unwrap();
    let metadata = generator.metadata.clone();
    let key = metadata.name.unwrap();

    if let Some(value) = inner_state.get(&key) {
        if value == &metadata.generation.unwrap() {
            // no need to update the value
            info!("no need to update the status");
            return Ok(())
        }
    }

    // otherwise update the state...
    inner_state.insert(String::from(&key), metadata.generation.unwrap());

    // set the data in the global var...
    let status = DecryptorStatus::new(
        SyncStatus::Sync,
          None,
          "foo".to_owned(),
          generator.status.clone(),
          "foo-bar".to_owned()
    );

    // retrieve the decryptor object to patch it
    let curr_decryptor = Api::<Decryptor>::namespaced(
        client.clone(),
        generator.namespace().as_ref().unwrap()
    );

    let mut curr_decryptor_status = curr_decryptor.get_status(&key).await?;
    curr_decryptor_status.status = Some(status);

    info!("Update status...");

    curr_decryptor.replace_status(
        &key,
        &PostParams::default(),
        serde_json::to_vec(&curr_decryptor_status).unwrap()
    ).await?;

    info!("Status has been updated");

    Ok(())
}

/// Create a controller which will watch the Decryptor resource
/// The implementation is based on:
///     - https://github.com/kube-rs/kube-rs/blob/bf3b248f0c96b229863e0bff510fdf118efd2381/examples/configmapgen_controller.rs
pub async fn boostrap_controller(state: State) -> Result<(), Error> {
    info!("Starting up the controller...");
    info!("Initializing client");
    let client = Client::try_default().await?;
    
    // Watch the Decryptor ressources
    let api: Api<Decryptor> = Api::all(client.clone());
    let watcher = watcher(api, ListParams::default());

    let mut apply_events = try_flatten_applied(watcher).boxed_local();
    while let Some(dec) = apply_events.try_next().await.unwrap() {
        // increase the ref counter
        reconcile(dec, client.clone(), state.clone()).await?;
    }
    
    Ok(())
}