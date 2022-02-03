use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[macro_use]
extern crate log;

mod controller;
mod error;

type State = Arc<Mutex<HashMap<String, i64>>>;

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

    let state = Arc::new(Mutex::new(HashMap::new()));
    controller::boostrap_controller(state).await?;

    Ok(())
}
