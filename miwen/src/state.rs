use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::err::Error;

pub type State = Arc<Mutex<HashMap<String, i64>>>;

/// Generate a new State
pub fn generate_new_state() -> State {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Check whenever the generation id exist in the state
/// If it does not exist / different, then adding / updating the value in the state
/// 
/// # Arguments
/// * `state` - State
/// * `key` - String
/// * `value` - i64
pub fn upsert_state(state: State, key: String, value: i64) -> Result<bool, Error> {
    let mut state = state
        .lock()
        .map_err(|_| Error::Watch("Unable to acquired lock".to_owned()))?;
    
    if let Some(inner) = state.get(&key) {
        if inner == &value {
            return Ok(true)
        }
    }

    // otherwise update the state...
    state.insert(String::from(&key), value);

    Ok(false)
}

/// Check if the CRD is registered in the state
/// This is gonna be used to trigger a grpc call to pull the repository
/// 
/// # Arguments
/// * `state` - State
/// * `key` - &str
pub fn is_registered(state: State, key: &str) -> Result<bool, Error> {
    let state = state.lock()
        .map_err(|_| Error::Watch("Unable to acquired the lock".to_owned()))?;

    if state.contains_key(key) {
        return Ok(true);
    }

    Ok(false)
}

/// Delete an item in the state. This case is used when a CRD is delete. w/o removing the item, when a crd containing the sma
/// name is applied. The crd might not take into account the update
/// 
/// # Arguments
/// * `state` - State
/// * `key` - &str
pub fn delete_item_in_state(state: State, key: &str) -> Result<(), Error> {
    let mut state = state.lock()
        .map_err(|_| Error::Watch("Unable to acquired the lock".to_owned()))?;

    state.remove(key);

    Ok(())
}