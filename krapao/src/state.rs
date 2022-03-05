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

impl List {
    /// Read the existing state stored in ~/workspace/repo/list.json
    fn read_persistent_state() -> Result<List, Error> {
        let mut file_path = home_dir().unwrap_or_default();
        file_path.push(REPO_PATH);
        file_path.push(REPO_FILE_PATH);
    
        let saved_state = fs::read(&file_path)?;
        let list: List = serde_json::from_slice(&saved_state)?;
        
        Ok(list)
    }

    /// Create a new empty state
    fn create_new_empty_state() -> Result<List, Error> {
        let default = List::default();
        default.save_list_to_persistent_state()?;

        Ok(default)
    }

    /// Save the persistent state in the ~/workspace/repo/list.json
    /// 
    /// # Arguments
    /// * `&self` - &Self
    fn save_list_to_persistent_state(&self) -> Result<(), Error> {
        let mut file_path = home_dir().unwrap_or_default();
        file_path.push(REPO_PATH);
        file_path.push(REPO_FILE_PATH);

        let json = serde_json::to_string_pretty(&self)?;
        fs::write(file_path, json)?;
        
        Ok(())
    }
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

    // import existing state in a config file
    let saved_state = match List::read_persistent_state() {
        Ok(res) => res,
        Err(_) => {
            // if the file could not be read, it means that it does not exist. So create it
            List::create_new_empty_state()?
        }
    };

    if let Some(existing_state) = saved_state.repositories {
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
    let mut list: List = List::read_persistent_state()?;

    if let Some(existing_state) = list.repositories.as_mut() {
        existing_state.insert(config.repo_uri.clone(), config);
    } else {
        let mut map = HashMap::new();
        map.insert(config.repo_uri.clone(), config);

        list.repositories = Some(map);
    }

    // now encrypt back the repo
    list.save_list_to_persistent_state()
}

/// Remove a config from the persistent state. This helps krapao
/// to not pull changes from the repo again
/// 
/// # Arguments
/// * `key` - &str
pub fn remove_repo_from_persistent_state(key: &str) -> Result<(), Error> {
    let mut list: List = List::read_persistent_state()?;

    if let Some(existing_state) = list.repositories.as_mut() {
        existing_state.remove(key);
    }

    list.save_list_to_persistent_state()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::repo::config::Credentials;

    use super::*;

    #[test]
    fn expect_to_create_state() {
        let state = create_state();
        assert!(state.is_ok());

        // test if the file exist
        let list = List::read_persistent_state();
        assert!(list.is_ok());
    }

    #[test]
    fn expect_to_save_new_config() {
        let repo_uri = "https://github.com/shigedangao/maskiedoc.git";
        let credentials = Credentials::Empty;
        let config = GitConfig::new(
            credentials, 
            repo_uri, 
            PathBuf::new()
        ).unwrap();
        
        let res = save_new_repo_in_persistent_state(config);
        assert!(res.is_ok());

        // get the list and we're gonna check if the repo exist
        let list = List::read_persistent_state().unwrap();
        let repos = list.repositories.unwrap();

        let maskiedoc = repos.get(repo_uri);
        assert!(maskiedoc.is_some());

        // remove the maskiedoc from the state
        let res = remove_repo_from_persistent_state(repo_uri);
        assert!(res.is_ok());
    }
}