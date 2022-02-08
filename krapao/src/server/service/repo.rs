use tonic::{async_trait, Response, Status, Request};
use self::proto::{
    repo_service_server::RepoService,
    Payload,
    Response as ProtoResponse,
};
use crate::repo;
use crate::env::Env;
use crate::state;
use crate::err::Error;

pub mod proto {
    tonic::include_proto!("repository");
}

#[derive(Debug, Default, Clone)]
pub struct RepoHandler {
    pub state: state::State
}

#[async_trait]
impl RepoService for RepoHandler {
    /// Configure a new repository in the krapao project.
    /// We're:
    ///     - Add the repository in the list of repos to listen
    ///     - Store the repository in a persistent state & tempo state
    ///     - Clone the repository
    ///     - Store the state which can be used by an async task run in parallel with the gRPC server
    /// 
    /// # Arguments
    /// * `&self` - Self
    /// * `request` - Request<Payload>
    async fn set_repository(
        &self,
        request: Request<Payload>
    ) -> Result<Response<ProtoResponse>, Status> {
        let input = request.into_inner();
        // retrieve the env from the request
        let env = Env::from(input);
        // retrieve the state
        let mut state = self.state.lock()
            .map_err(|err| Error::Server(err.to_string()))?;

        // if the state is already contain the repository then we don't need to clone it again
        if state.contains_key(&env.repository) {
            return Ok(Response::new(ProtoResponse {
                done: true
            }))
        }

        // maybe do this async ?
        let config = repo::initialize_git(&env)?;
        
        // add the new git config in the state
        state.insert(env.repository, config.clone());
        state::save_new_repo_in_persistent_state(config)?;

        Ok(Response::new(ProtoResponse {
            done: true
        }))
        // in this case we're going to trigger the creation of a new repo
    }

    /// Delete a repository from the list of repository
    ///     - Remove the repo from the list of repo
    ///     - Remove the repo from the persistent state
    ///     - Delete the repository
    /// 
    /// # Arguments
    /// * `self` - Self
    /// * `request` - Request<Payload>
    async fn delete_repository(
        &self,
        request: Request<Payload>
    ) -> Result<Response<ProtoResponse>, Status> {
        let input = request.into_inner();
        // convert the input as an Env
        let env = Env::from(input);
        // get the state
        let mut state = self.state.lock()
            .map_err(|err| Error::Server(err.to_string()))?;

        if state.contains_key(&env.repository) {
            let config = state.remove(&env.repository);
            if let Some(git) = config {
                // remove the value from the persistent state
                state::remove_repo_from_persistent_state(&env.repository)?;
                git.delete_repository()?;
            }
        } else {
            info!("No repository to delete");
        }

        Ok(Response::new(ProtoResponse {
            done: true,
        }))
    }
}
