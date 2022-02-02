use std::{sync::Arc, collections::BTreeMap};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::{core::ObjectMeta, Resource};
use crd::{Decryptor, DecryptorStatus};
use err::Error;

pub mod crd;
pub mod err;

fn get_owner_reference_from_meta(meta: ObjectMeta) -> Result<Vec<OwnerReference>, Error> {
    let owner = OwnerReference {
        controller: Some(true),
        api_version: Decryptor::api_version(&()).to_string(),
        kind: Decryptor::kind(&()).to_string(),
        name: meta.name.ok_or(Error::MissingMetadata("name".to_owned()))?,
        uid: meta.uid.ok_or(Error::MissingMetadata("uid".to_owned()))?,
        ..OwnerReference::default()
    };

    Ok(vec![owner])
}

impl Decryptor {
    pub fn from_generator(gen: Arc<Decryptor>) -> Result<Decryptor, Error> {
        let mut label = BTreeMap::new();
        label.insert("lol".to_owned(), "o".to_owned());
        
        Ok(Decryptor {
            metadata: ObjectMeta {
                name: gen.metadata.name.clone(),
                owner_references: Some(get_owner_reference_from_meta(gen.metadata.clone())?),
                labels: Some(label),
                ..ObjectMeta::default()
            },
            spec: gen.spec.clone(),
            status: Some(DecryptorStatus {
                message: "nah !".to_owned()
            })
        })
    }
}