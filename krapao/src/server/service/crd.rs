use tonic::{async_trait, Response, Status, Request};
use self::proto::{
    crd_service_server::CrdService,
    Response as ProtoResponse,
    Payload,
    payload::Provider as ProtoProvider
};
use crate::state;
use crate::err::Error;
use crate::sops;
use crate::auth::Provider;

// Constant
const REPO_NOT_EXIST_ERR_MSG: &str = "Repository does not exist";
const MISSING_PROVIDER_ERR_MSG: &str = "Unable to define the provider uses to decrypt sops";

pub mod proto {
    tonic::include_proto!("crd");
}

#[derive(Debug, Default, Clone)]
pub struct CrdHandler {
    pub state: state::State
}

#[async_trait]
impl CrdService for CrdHandler {
    async fn render(
        &self,
        request: Request<Payload>
    ) -> Result<Response<ProtoResponse>, Status> {
        let input = request.into_inner();
        // get a lock and retrieve the state
        let guard = self.state.lock()
            .map_err(|err| Error::Server(err.to_string()))?;

        let config = guard.get(&input.repository)
            .ok_or_else(|| Error::Server(REPO_NOT_EXIST_ERR_MSG.to_owned()))?;

        // authenticate with one of the provider
        let input_provider = ProtoProvider::from_i32(input.provider)
            .ok_or_else(|| Error::Server(MISSING_PROVIDER_ERR_MSG.to_owned()))?;

        let provider = Provider::from(input_provider);
        provider.authenticate(&input.credentials)?;

        let res = sops::decrypt_file(config, &input.file_to_decrypt, &input.sops_file_path)?;
        let commit_hash = config.get_commit_hash();

        Ok(Response::new(ProtoResponse {
            resource: res,
            commit_hash
        }))
    }
}