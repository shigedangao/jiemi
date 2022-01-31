use std::time::Duration;
use std::thread::{self, JoinHandle};
use crate::err::Error;
use super::GitConfig;

// constant
// we use quicker timeout for debugging purposes
#[cfg(debug_assertions)]
const MIN_REFRESH_INTERVAL: u64 = 10;
#[cfg(not(debug_assertions))]
const MIN_REFRESH_INTERVAL: u64 = 120;
const MAX_RETRY: u8 = 5;

/// Watch repository by pulling the repository from time to time
/// This method run a thread in a loop. The loop will only exist in these two cases:
///     - interval time lower than the minimum authorized value
///     - retry interval exceeding the maximum allowable value
/// 
/// # Arguments
/// * `config` - GitConfig
/// * `interval` - Option<u64>
/// * `target` - String
pub fn watch_repository(config: GitConfig, interval: Option<u64>, target: String) -> Result<(), Error> {
    let interval = interval.unwrap_or(MIN_REFRESH_INTERVAL);
    if interval < MIN_REFRESH_INTERVAL {
        return Err(Error::RefreshDuration);
    }

    let handle: JoinHandle<Result<(), Error>> = thread::spawn(move || {
        let mut retry = 0;
        loop {
            let res = config.pull(target.to_owned());
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
                info!("Retrying to pull repository for {retry}th time...");
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Credentials;

    #[test]
    fn expect_watch_to_return_error() {
        let credentials = Credentials::Empty;
        let handle = GitConfig::new(credentials, "https://github.com/shigedangao/graphie.git", "").unwrap();

        let res = watch_repository(handle, Some(0), "".to_owned());
        assert_eq!(res.unwrap_err(), Error::RefreshDuration);
    }

    #[test]
    fn expect_watch_to_fail() {
        let credentials = Credentials::Empty;
        let handle = GitConfig::new(credentials, "https://github.com/shigedangao/graphie.git", "").unwrap();

        let res = watch_repository(handle, None, "../../foo".to_owned());
        assert_eq!(res.unwrap_err(), Error::MaxPullRetry);
    }
}