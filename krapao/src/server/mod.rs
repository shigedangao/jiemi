use tonic::transport::Server;
use crate::err::Error;
use crate::state::State;
use self::service::repo::{
    proto::repo_service_server::RepoServiceServer,
    RepoHandler
};
use self::service::crd::{
    proto::crd_service_server::CrdServiceServer,
    CrdHandler
};

pub mod service;

/// Initialize the gRPC server
/// The server is used to communicate with the controller
/// 
/// # Arguments
/// * `state` - State
pub async fn bootstrap_server(state: State) -> Result<(), Error> {
    info!("Gearing up the krapao server");
    let addr = match std::env::var_os("MODE") {
        Some(res) => {
            if res == "release" {
                "0.0.0.0"
            } else {
                "127.0.0.1"
            }
        },
        None => "127.0.0.1"
    };

    let addr = format!("{addr}:50208").parse()
        .map_err(|_| Error::Server("Unable to allocate address".to_owned()))?;

    Server::builder()
        .add_service(RepoServiceServer::new(RepoHandler {
            state: state.clone()
        }))
        .add_service(CrdServiceServer::new(CrdHandler {
            state: state.clone()
        }))
        .serve(addr)
        .await?;

    Ok(())
}