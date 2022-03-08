use std::{env, fs};
use crate::err::Error;

// Constant
const ENV_NAME: &str = "GOOGLE_APPLICATION_CREDENTIALS";
const FILENAME: &str = "../credentials.json";

/// Set the authentication file for Google Cloud Project
/// This is going to be used by sops in order to decrypt the file
/// 
/// # Arguments
/// * `credentials` - &str
pub fn set_authentication_file_for_gcp(credentials: &str) -> Result<(), Error> {
    // writing the configuration file
    fs::write("../credentials.json", credentials)
        .map_err(|err| Error::ProviderAuth(err.to_string()))?;
        
    env::set_var(ENV_NAME, FILENAME);

    Ok(())
}