use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

#[derive(Debug, JsonSchema, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub url: String,
    pub credentials: Option<RepositoryCredentials>
}

#[derive(Debug, JsonSchema, Clone, Serialize, Deserialize)]
pub struct RepositoryCredentials {
    pub username: Option<GenericConfig>,
    pub token: Option<GenericConfig>,
    pub ssh: Option<GenericConfig>
}

impl RepositoryCredentials {
    fn get_username_value() -> Option<String> {}
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct GenericConfig {
    secret_name: Option<String>,
    key: Option<String>,
    literal: Option<String>
}

impl GenericConfig {
    
}