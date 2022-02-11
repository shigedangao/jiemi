use kube::{
    Api,
    Client,
    core::{
        DynamicObject,
        GroupVersionKind,
        ApiResource, ObjectMeta,
    },
    api::{
        PatchParams,
        Patch,
        PostParams
    },
};
use serde::Deserialize;
use crate::err::Error;

// Constant
const API_GROUP_SPLIT: &str = "/";

#[derive(Deserialize, Debug)]
struct GvkWrapper {
    #[serde(rename = "apiVersion")]
    api_version: String,
    kind: String,
    metadata: ObjectMeta
}

impl GvkWrapper {
    fn get_gkv(&self) -> GroupVersionKind {
        let splitted_group = self.api_version.split_once(API_GROUP_SPLIT);

        match splitted_group {
            Some((group, version)) => GroupVersionKind {
                group: group.to_owned(),
                version: version.to_owned(),
                kind: self.kind.clone()
            },
            None => GroupVersionKind {
                // default group is core and does not have a name...
                group: "".to_owned(),
                version: self.api_version.clone(),
                kind: self.kind.clone()
            }
        }
    }

    fn get_name(self) -> Option<String> {
        self.metadata.name
    }
}

async fn create_resource(api: Api<DynamicObject>, tmpl: &str) -> Result<(), Error> {
    let patch: DynamicObject = serde_yaml::from_str(&tmpl)?;
    api.create(&PostParams::default(), &patch).await?;

    Ok(())
}

async fn patch_resource(api: Api<DynamicObject>, name: &str, tmpl: &str) -> Result<(), Error> {
    let patch: DynamicObject = serde_yaml::from_str(&tmpl)?;
    let res = api.patch(
        &name, 
        &PatchParams::apply("miwen").force(),
         &Patch::Apply(&patch)
    ).await;

    match res {
        Ok(_) => {
            info!("Resource {name} has been successfully synchronized");
            Ok(())
        },
        Err(err) => {
            error!("{err:?}");
            Err(Error::from(err))
        }
    }
}

pub async fn apply_rendered_object(tmpl: String, client: &Client, ns: &str) -> Result<(), Error> {
    // Get the gvk struct from the rendered yaml
    let gvk_wrapper: GvkWrapper = serde_yaml::from_str(&tmpl)?;
    let gvk = gvk_wrapper.get_gkv();
    // create an api_resource from the gvk
    let api_resource = ApiResource::from_gvk(&gvk);
    // get a dynamic object
    let api: Api<DynamicObject> = Api::namespaced_with(client.clone(), ns, &api_resource);

    let res_name = gvk_wrapper.get_name()
        .ok_or(Error::Watch("❌ Provided resource does not have a name".to_owned()))?;

    // get a dynamic object to retrieve the metadata...
    let res = api.get(&res_name).await;
    match res {
        Ok(_) => patch_resource(api, &res_name, &tmpl).await,
        Err(_) => create_resource(api, &tmpl).await
    }
}