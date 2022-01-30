use std::time::Duration;
use std::thread::{self, JoinHandle};
use crate::err::Error;
use super::GitConfig;

// constant
const MIN_REFRESH_INTERVAL: u64 = 180;
const MAX_RETRY: u8 = 20;

/// Watch repository by pulling the repository from time to time
/// This method run a thread in a loop. The loop will only exist in these two cases:
///     - interval time lower than the minimum authorized value
///     - retry interval exceeding the maximum allowable value
/// 
/// # Arguments
/// * `config` - GitConfig
/// * `interval` - u64
pub fn watch_repository(config: GitConfig, interval: u64) -> Result<(), Error> {    
    let handle: JoinHandle<Result<(), Error>> = thread::spawn(move || {
        let mut retry = 0;
        if interval < MIN_REFRESH_INTERVAL {
            return Err(Error::RefreshDuration);
        }

        loop {
            let res = config.pull();
            if res.is_ok() {
                retry = 0;
            }
            // in the case of an error we're only going to display the message
            // and retry until max retry value
            if let Err(err) = res {
                if retry > MAX_RETRY {
                    return Err(Error::MaxPullRetry);
                }
                retry += 1;
                error!("{err}");
            }

            thread::sleep(Duration::from_secs(interval));
        }
    });

    let res = handle.join();
    match res {
        Ok(res) => res,
        Err(err) => {
            match err.downcast_ref::<String>() {
                Some(msg) => Err(Error::Pull(msg.to_string())),
                None => Err(Error::Pull("An unexpected unknown error happened while pulling the repository".to_string()))
            }
        }
    }
}