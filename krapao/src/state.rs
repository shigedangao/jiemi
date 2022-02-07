use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::repo::config::GitConfig;

pub type State = Arc<Mutex<HashMap<String, GitConfig>>>;

/// Create a new State is used to store the set of GitConfig
/// This state will be used by a different async task which will synchronize
/// the git repository
pub fn create_state() -> State {
    Arc::new(Mutex::new(HashMap::new()))
}