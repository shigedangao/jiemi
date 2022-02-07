use color_eyre::eyre::Result;

#[macro_use]
extern crate log;

mod repo;
mod err;
mod env;
mod helper;
mod server;
mod state;

/// Setup different logging & debugging services
fn setup() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "debug");
    }

    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "full");
    }

    // init the env loggger
    env_logger::init();
    color_eyre::install()
}

#[tokio::main]
async fn main() -> Result<()> {
    setup()?;
    // create the state
    let state = state::create_state();

    // bootstrap the server
    server::bootstrap_server(state).await?;

    Ok(())
}
