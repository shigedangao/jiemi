use std::{sync::Arc, time::Duration};
use kube::{
    Client,
    runtime::controller::{Controller, Context, ReconcilerAction}, Api, api::{ListParams, PatchParams, Patch},
    Error as KubeError, ResourceExt
};
use crate::error::Error;
use gen::crd::Decryptor;
use futures::StreamExt;

struct Data {
    client: Client
}

async fn reconcile(generator: Arc<Decryptor>, ctx: Context<Data>) -> Result<ReconcilerAction, Error> {
    let client = ctx.get_ref().client.clone();
    
    // create a new Decryptor struct from the arc
    let decryptor = Decryptor::from_generator(generator.clone())?;
    println!("{:?}", decryptor);

    // retrieve the decryptor object to patch it
    let curr_decryptor = Api::<Decryptor>::namespaced(
        client.clone(),
        generator.namespace().as_ref().unwrap()
    );

    curr_decryptor.patch(
        decryptor.metadata.name.as_ref().unwrap(),
        &PatchParams::apply("decryptor.jiemi.cr"),
        &Patch::Apply(&decryptor)
    ).await?;

    Ok(ReconcilerAction {
        requeue_after: Some(Duration::from_secs(300))
    })
}

fn error_policy(error: &Error, _ctx: Context<Data>) -> ReconcilerAction {
    error!("{:?}", error);
    ReconcilerAction {
        requeue_after: Some(Duration::from_secs(300))
    }
}

/// Create a controller which will watch the Decryptor resource
/// The implementation is based on:
///     - https://github.com/kube-rs/kube-rs/blob/bf3b248f0c96b229863e0bff510fdf118efd2381/examples/configmapgen_controller.rs
pub async fn boostrap_controller() -> Result<(), Error> {
    info!("Starting up the controller...");
    info!("Initializing client");
    let client = Client::try_default().await?;
    
    // Watch the Decryptor ressources
    let decryptor: Api<Decryptor> = Api::all(client.clone());
    let ctx = Context::new(Data { client });

    Controller::new(decryptor, ListParams::default())
        .run(reconcile, error_policy, ctx)
        .for_each(|res| async move {
            match res {
                Ok(o) => { info!("{:?}", o) },
                Err(err) => { error!("{err}") }
            }
        })
        .await;

    Ok(())
}