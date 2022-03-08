use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::err::Error;

// Constant
const LOCK_ERR_MSG: &str = "Unable to acquired lock";

pub type State = Arc<Mutex<HashMap<String, i64>>>;

/// Generate a new State
/// 
/// The state is used to stored the list of CRD that has been registered when a user used the command
/// kubectl apply -f <crd>
/// As we're also updating the CRD. Using a state ensure us that this won't create an infinite loop of update
pub fn generate_new_state() -> State {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Check whenever the generation id exist in the state
/// If it does not exist / different, then adding / updating the value in the state
/// 
/// # Arguments
/// * `state` - State
/// * `key` - &str
/// * `value` - i64
pub fn upsert_state(state: State, key: &str, value: i64) -> Result<bool, Error> {
    let mut state = state
        .lock()
        .map_err(|_| Error::Watch(LOCK_ERR_MSG.to_owned()))?;
    
    if let Some(inner) = state.get(key) {
        if inner == &value {
            return Ok(true)
        }
    }

    // otherwise update the state...
    state.insert(String::from(key), value);

    Ok(false)
}

/// Check if the CRD is registered in the state
/// This is gonna be used to trigger a grpc call to pull the repository if a new CRD is founded
/// Why ? If a new CRD is pushed we want to clone it's associated repository only once.
/// 
/// # Arguments
/// * `state` - State
/// * `key` - &str
pub fn is_registered(state: State, key: &str) -> Result<bool, Error> {
    let state = state.lock()
        .map_err(|_| Error::Watch(LOCK_ERR_MSG.to_owned()))?;

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
        .map_err(|_| Error::Watch(LOCK_ERR_MSG.to_owned()))?;

    state.remove(key);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expect_to_add_new_element_in_state() {
        let state = generate_new_state();
        let res = upsert_state(state, "foo", 1).unwrap();
        
        assert_eq!(res, false);
    }

    #[test]
    fn expect_upsert_to_return_false() {
        let state = generate_new_state();

        let res = upsert_state(state.clone(), "foo", 1).unwrap();
        assert_eq!(res, false);

        let res = upsert_state(state, "foo", 1).unwrap();
        assert_eq!(res, true);
    }

    #[test]
    fn expect_to_update_existing_element_in_state() {
        let state = generate_new_state();
        upsert_state(state.clone(), "foo", 1).unwrap();
        upsert_state(state.clone(), "foo", 2).unwrap();

        // get the value and check it
        let map = state.lock().unwrap();
        let item = map.get("foo");

        assert_eq!(item.unwrap().to_owned(), 2);
    }

    #[test]
    fn expect_state_to_be_registered() {
        let state = generate_new_state();
        upsert_state(state.clone(), "foo", 1).unwrap();
        
        let res = is_registered(state, "foo").unwrap();
        assert_eq!(res, true);
    }

    #[test]
    fn expect_state_to_not_be_registered() {
        let state = generate_new_state();
        let res = is_registered(state, "foo").unwrap();
        
        assert_eq!(res, false);
    }

    #[test]
    fn expect_to_delete_item_in_state() {
        let state = generate_new_state();
        upsert_state(state.clone(), "foo", 1).unwrap();

        let res = delete_item_in_state(state.clone(), "foo");
        assert!(res.is_ok());

        let map = state.lock().unwrap();
        let item = map.get("foo");
        assert!(item.is_none());
    }
}