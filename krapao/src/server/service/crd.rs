use tonic::{async_trait, Response, Status, Request};
use self::proto::{
    crd_service_server::CrdService,
    Response as ProtoResponse,
    Payload
};
use crate::state;
use crate::err::Error;
use crate::sops;

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
            .ok_or(Error::Server("Repository does not exist".to_owned()))?;

        let res = sops::decrypt_file(&config, &input.file_to_decrypt, &input.sops_file_path)?;
        let commit_hash = config.get_commit_hash();

        Ok(Response::new(ProtoResponse {
            resource: res,
            commit_hash
        }))
    }
}