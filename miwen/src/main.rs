use color_eyre::Result;

#[macro_use]
extern crate log;

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

fn main() -> Result<()> {
    setup()?;
    println!("Hello, world!");

    Ok(())
}
