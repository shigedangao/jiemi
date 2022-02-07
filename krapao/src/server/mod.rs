use tonic::transport::Server;
use crate::err::Error;
use crate::state::State;
use self::service::repo::{
    proto::repo_service_server::RepoServiceServer,
    RepoHandler
};

pub mod service;

/// Initialize the gRPC server
/// The server is used to communicate with the controller
pub async fn bootstrap_server(state: State) -> Result<(), Error> {
    info!("Gearing up the krapao server");
    let addr = "[::1]:50208".parse()
        .map_err(|_| Error::Server("Unable to allocate address".to_owned()))?;

    Server::builder()
        .add_service(RepoServiceServer::new(RepoHandler {
            state: state.clone()
        }))
        .serve(addr)
        .await?;

    Ok(())
}