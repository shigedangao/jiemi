use std::{sync::Arc, time::Duration};
use kube::{
    Client,
    runtime::controller::{Controller, Context, ReconcilerAction}, Api, api::ListParams,
    Error as KubeError
};
use crate::error::Error;
use gen::crd::Decryptor;
use futures::StreamExt;

struct Data {
    client: Client
}

async fn reconcile(generator: Arc<Decryptor>, ctx: Context<Data>) -> Result<ReconcilerAction, KubeError> {
    Ok(ReconcilerAction {
        requeue_after: Some(Duration::from_secs(300))
    })
}

fn error_policy(error: &KubeError, _ctx: Context<Data>) -> ReconcilerAction {
    ReconcilerAction {
        requeue_after: Some(Duration::from_secs(300))
    }
}

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