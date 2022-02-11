#[macro_use]
extern crate log;

mod controller;
mod err;
mod state;
mod client;

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
    // @TODO if it crash then we should restart the task...
    // Create a counter with a max time value
    controller::boostrap_watcher(state).await?;

    Ok(())
}
