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
const MISSING_NAME_ERR: &str = "❌ Provided resource does not have a name";

#[derive(Deserialize, Debug)]
struct GvkWrapper {
    #[serde(rename = "apiVersion")]
    api_version: String,
    kind: String,
    metadata: ObjectMeta
}

impl GvkWrapper {
    /// Retrieve the GVK from the wrapper
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

    /// Get the name of the kubernetes resource
    fn get_name(self) -> Option<String> {
        self.metadata.name
    }
}

/// Create a resource based on the DynamicObject
/// 
/// # Arguments
/// * `api` - Api<DynamicObject>
/// * `patch` - DynamicObject
async fn create_resource(api: Api<DynamicObject>, patch: DynamicObject) -> Result<(), Error> {
    api.create(&PostParams::default(), &patch).await?;
    info!("📝 Resource has been created");

    Ok(())
}

/// Patch a Kubernetes resource with the dynamic object
/// 
/// # Arguments
/// * `api` - Api<DynamicObject>
/// * `name` - &str
/// * `patch` - DynamicObject
async fn patch_resource(api: Api<DynamicObject>, name: &str, patch: DynamicObject) -> Result<(), Error> {
    let res = api.patch(
        name, 
        &PatchParams::apply("miwen").force(),
         &Patch::Apply(&patch)
    ).await;

    match res {
        Ok(_) => {
            info!("🖌️ Resource {name} has been successfully synchronized");
            Ok(())
        },
        Err(err) => {
            error!("{err:?}");
            Err(Error::from(err))
        }
    }
}

/// Apply the rendered template in the Kubernetes cluster
/// Because we couldn't applied YAML straight away. We need to retrieve the resource type
/// to create the DynamicObject. We need to retrieve the:
///     - GVK
///     - Create an apiResource from the GVK
///     - name of the resource
/// 
/// If the resource already exist, then we're going to patch it. Otherwise we'll create the resource
/// 
/// # Arguments
/// * `tmpl` - String
/// * `client` - &Client
/// * `ns` - &str
pub async fn apply_rendered_object(tmpl: String, client: &Client, ns: &str) -> Result<(), Error> {
    // Get the gvk struct from the rendered yaml
    let gvk_wrapper: GvkWrapper = serde_yaml::from_str(&tmpl)?;
    let gvk = gvk_wrapper.get_gkv();
    // create an api_resource from the gvk
    let api_resource = ApiResource::from_gvk(&gvk);
    // get a dynamic object
    let api: Api<DynamicObject> = Api::namespaced_with(client.clone(), ns, &api_resource);

    let res_name = gvk_wrapper.get_name()
        .ok_or_else(|| Error::Watch(MISSING_NAME_ERR.to_owned()))?;

    // get a dynamic object to retrieve the metadata...
    let res = api.get(&res_name).await;
    let patch: DynamicObject = serde_yaml::from_str(&tmpl)?;
    match res {
        Ok(_) => patch_resource(api, &res_name, patch).await,
        Err(_) => create_resource(api, patch).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::core::v1::ConfigMap;
    
    #[test]
    fn expect_to_get_gvk_with_group() {
        let wrapper = GvkWrapper {
            api_version: "jiemi.cr/v1alpha1".to_owned(),
            kind: "Decryptor".to_owned(),
            metadata: ObjectMeta::default()
        };

        let gvk = wrapper.get_gkv();
        
        assert_eq!(gvk.group, "jiemi.cr");
        assert_eq!(gvk.version, "v1alpha1");
        assert_eq!(gvk.kind, "Decryptor");
    }

    #[test]
    fn expect_to_get_gvk_wo_group() {
        let wrapper = GvkWrapper {
            api_version: "v1".to_owned(),
            kind: "Secret".to_owned(),
            metadata: ObjectMeta::default()
        };

        let gvk = wrapper.get_gkv();
        
        assert_eq!(gvk.group, "");
        assert_eq!(gvk.version, "v1");
        assert_eq!(gvk.kind, "Secret");
    }

    #[test]
    fn expect_to_get_name() {
        let mut meta = ObjectMeta::default();
        meta.name = Some("foo".to_owned());

        let wrapper = GvkWrapper {
            api_version: "v1".to_owned(),
            kind: "Secret".to_owned(),
            metadata: meta
        };

        let name = wrapper.get_name().unwrap();
        assert_eq!(name, "foo");
    }

    #[tokio::test]
    async fn expect_to_apply_rendered_object() {
        let configmap = r#"
        apiVersion: v1
        kind: ConfigMap
        metadata:
          name: miwen-unit-test
        data:
          player_initial_lives: "3"
          ui_properties_file_name: "user-interface.properties"
        "#;

        let client = Client::try_default().await.unwrap();

        // creation
        let res = apply_rendered_object(configmap.to_owned(), &client, "default").await;
        assert!(res.is_ok());

        let updated_configmap = r#"
        apiVersion: v1
        kind: ConfigMap
        metadata:
          name: miwen-unit-test
        data:
          player_initial_lives: "5"
          ui_properties_file_name: "user-interface.properties"
        "#;
        
        let res = apply_rendered_object(updated_configmap.to_owned(), &client, "default").await;
        assert!(res.is_ok());

        // Checking that the value is really 5
        let configmap: Api<ConfigMap> = Api::namespaced(client, "default");
        let miwen_unit_test_map = configmap.get("miwen-unit-test").await.unwrap();

        let data = miwen_unit_test_map.data.unwrap();
        assert_eq!(data.get("player_initial_lives").unwrap(), "5");
    }
}