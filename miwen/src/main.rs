#[macro_use]
extern crate log;

mod watcher;
mod err;
mod state;
mod client;
mod sync;

/// Setup different logging & debugging services
fn setup() -> color_eyre::Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "miwen=info");
    }

    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "full");
    }

    // init the env loggger
    env_logger::init();
    color_eyre::install()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup()?;

    let state = state::generate_new_state();
    tokio::try_join!(
        // Start the watcher which will react to any changes on the crd
        watcher::boostrap_watcher(state),
        // Start a sync loop which will sync the repo with the cluster
        sync::bootstrap_repo_sync()
    )?;

    Ok(())
}
