use color_eyre::eyre::Result;

#[macro_use]
extern crate log;

mod repo;
mod err;
mod env;
mod helper;
mod server;
mod state;
mod sync;

/// Setup different logging & debugging services
fn setup() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "krapao=debug");
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
    let state = state::create_state()?;

    // bootstrap the server
    let res = tokio::try_join!(
        server::bootstrap_server(state.clone()),
        sync::synchronize_repository(state.clone())
    );

    match res {
        Ok(_) => error!("Expect server / watcher to not stop"),
        Err(err) => error!("{}", err.to_string())
    }

    Ok(())
}
