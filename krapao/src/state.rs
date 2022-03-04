use std::sync::{Arc, Mutex};
use std::fs;
use std::collections::HashMap;
use dirs::home_dir;
use serde::{Serialize, Deserialize};
use crate::repo::config::GitConfig;
use crate::err::Error;
use crate::helper;

// Constant
const REPO_FILE_PATH: &str = "list.json";
const REPO_PATH: &str = "workspace/repo";

// Alias type
pub type State = Arc<Mutex<HashMap<String, GitConfig>>>;

#[derive(Debug, Serialize, Deserialize, Default)]
struct List {
    repositories: Option<HashMap<String, GitConfig>>
}

/// Create a new State is used to store the set of GitConfig
/// This state will be used by a different async task which will synchronize
/// the git repository
/// 
/// If an existing state exist. Then retrieve the state and use it. This 
pub fn create_state() -> Result<State, Error>  {
    let mut workspace_dir = home_dir().unwrap_or_default();
    // Create the dir if it does not exist
    workspace_dir.push(REPO_PATH);
    helper::create_path(&workspace_dir)?;

    let mut file_path = workspace_dir.clone();
    file_path.push(REPO_FILE_PATH);

    // import existing state in a config file
    let saved_state = match fs::read(&file_path) {
        Ok(res) => res,
        Err(_) => {
            // if the file could not be read, it means that it does not exist. So create it
            let list = List::default();
            let json = serde_json::to_string_pretty(&list)?;
            fs::write(file_path, &json)?;

            json.as_bytes().to_vec()
        }
    };

    let list: List = serde_json::from_slice(&saved_state)?;
    if let Some(existing_state) = list.repositories {
        return Ok(Arc::new(Mutex::new(existing_state)));
    }

    Ok(Arc::new(Mutex::new(HashMap::new())))
}

/// Save the new repo config in the persistent state. 
/// This enable us to not clone the repo again...
/// 
/// # Arguments
/// * `config` - GitConfig
pub fn save_new_repo_in_persistent_state(config: GitConfig) -> Result<(), Error> {
    let mut file_path = home_dir().unwrap_or_default();
    file_path.push(REPO_PATH);
    file_path.push(REPO_FILE_PATH);

    let saved_state = fs::read(&file_path)?;
    let mut list: List = serde_json::from_slice(&saved_state)?;

    if let Some(existing_state) = list.repositories.as_mut() {
        existing_state.insert(config.repo_uri.clone(), config);
    } else {
        let mut map = HashMap::new();
        map.insert(config.repo_uri.clone(), config);

        list.repositories = Some(map);
    }

    // now encrypt back the repo
    let json = serde_json::to_string_pretty(&list)?;
    fs::write(file_path, json)?;

    Ok(())
}

/// Remove a config from the persistent state. This helps krapao
/// to not pull changes from the repo again
/// 
/// # Arguments
/// * `key` - &str
pub fn remove_repo_from_persistent_state(key: &str) -> Result<(), Error> {
    let mut file_path = home_dir().unwrap_or_default();
    file_path.push(REPO_PATH);
    file_path.push(REPO_FILE_PATH);

    let saved_state = fs::read(&file_path)?;
    let mut list: List = serde_json::from_slice(&saved_state)?;

    if let Some(existing_state) = list.repositories.as_mut() {
        existing_state.remove(key);
    }

    let json = serde_json::to_string_pretty(&list)?;
    fs::write(file_path, json)?;
    
    Ok(())
}