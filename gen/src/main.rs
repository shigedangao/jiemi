use std::env;
use std::fs;
use color_eyre::Result;

pub mod crd;
mod err;
mod util;

/// Setup different logging & debugging services
fn setup() -> Result<()> {
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "full");
    }

    color_eyre::install()
}

fn main() -> Result<()> {
    setup()?;
    
    // retrieve a filename path if given
    // not using clap as we're only focusing on a single arg...
    let path = env::args().nth(1).unwrap_or(".".to_owned());
    let full_path = format!("{path}/crd.yaml");

    let spec = crd::generate_crd().expect("Expect to generate CRD");
    fs::write(&full_path, spec)?;
    println!("✅ CRD has been generated at the path {full_path}");

    Ok(())
}
