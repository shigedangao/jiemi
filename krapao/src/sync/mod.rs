use tokio::time::{sleep, Duration};
use crate::err::Error;
use crate::state::State;

// constant
const THREAD_SLEEP: u64 = 180;

/// Synchronize the list of repository which is stored in the State
/// Synchronization happened every 3min.
/// Each repo is updated in it's own thread. Sync is also done at the startup of krapao.
/// This is useful in case if the statefulset is restarting
/// 
/// # Arguments
/// * `state` - State
pub async fn synchronize_repository(state: State) -> Result<(), Error> {
    loop {
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

        // droping the mutex before sleeping the task to avoid creating deadlock
        drop(guard);
        sleep(Duration::from_secs(THREAD_SLEEP)).await;
    }
}