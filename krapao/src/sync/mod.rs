use tokio::time::{sleep, Duration};
use crate::err::Error;
use crate::state::State;

// constant
const THREAD_SLEEP: u64 = 180;

/// Synchronize the list of repository which is stored in the State
/// Synchronization happened every 3min.
/// Each repo is updated in it's own thread
/// 
/// # Arguments
/// * `state` - State
pub async fn synchronize_repository(state: State) -> Result<(), Error> {
    loop {
        // sleep is executed first to avoid keeping the guard too much time...
        sleep(Duration::from_secs(THREAD_SLEEP)).await;
        let guard = state.lock()
            .map_err(|err| Error::Sync(err.to_string()))?;

        for (_, config) in guard.clone().into_iter() {
            // create an async task which will pull the repository
            // the pull method will exit if the timeout exceed
            // join them altogether...
            tokio::spawn(async move {
                if let Err(err) = config.pull() {
                    error!("Error while pulling repository: {}", err.to_string());
                }
            });
        }
    }
}