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
    async fn set_repository(
        &self,
        request: Request<Payload>
    ) -> Result<Response<ProtoResponse>, Status> {
        let input = request.into_inner();
        // retrieve the env from the request
        let env = Env::from(input);

        info!("acquire lock");
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

    async fn delete_repository(
        &self,
        request: Request<Payload>
    ) -> Result<Response<ProtoResponse>, Status> {
        let _ = request.into_inner();

        Ok(Response::new(ProtoResponse {
            done: true,
        }))
    }
}
