use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::error::Error;

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
pub fn gen_id_exist_from_state(state: State, key: String, value: i64) -> Result<bool, Error> {
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