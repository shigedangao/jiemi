use gen::crd::DecryptorSpec;
use tonic::Request;
use crate::err::Error;
use self::proto::{
    repo_service_client::RepoServiceClient,
    Payload,
    Credentials
};

mod proto {
    tonic::include_proto!("repository");
}

pub async fn dispatch_clone_repository(spec: &DecryptorSpec) -> Result<(), Error> {
    let mut client = RepoServiceClient::connect("http://[::1]:50208").await?;
    // request to grpc server
    let req = Request::new(Payload {
        url: spec.source.repository.url.clone(),
        cred: None
    });
    
    let res = client.set_repository(req).await;

    Ok(())
}