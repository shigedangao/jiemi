use std::process::Command;
use std::fs;
use serde::Serialize;
use crate::err::Error;

// Constant
const AWS_CREDENTIALS_PATH: &str = "~/.aws/credentials";
const AWS_REGION_PATH: &str = "~/.aws/config";
const FAILURE_AUTH_ERR: &str = "Authentication test failed with AWS. Creds might be wrong";

#[derive(Serialize)]
#[serde(rename = "default")]
struct AwsCredentials {
    aws_access_key_id: String,
    aws_secret_access_key: String
}

#[derive(Serialize)]
#[serde(rename = "default")]
struct AwsRegion {
    region: String
}

/// Write Aws Config
/// 
/// # Arguments
/// * `access_key` - String
/// * `secret_key` - String
/// * `region` - String
fn write_aws_config(access_key: String, secret_key: String, region: String) -> Result<(), Error> {
    let profile = AwsCredentials {
        aws_access_key_id: access_key,
        aws_secret_access_key: secret_key
    };

    let toml = toml::to_string(&profile)?;
    fs::write(AWS_CREDENTIALS_PATH, toml)?;

    let reg = AwsRegion {
        region
    };

    let toml = toml::to_string(&reg)?;
    fs::write(AWS_REGION_PATH, &toml)?;

    Ok(())
}

/// Test authentication with aws command
fn test_authentication() -> Result<(), Error> {
    let status = Command::new("aws")
        .arg("sts")
        .arg("get-caller-identity")
        .status()?;

    if !status.success() {
        return Err(Error::Auth(FAILURE_AUTH_ERR.to_owned()))
    }

    Ok(())
}

/// Authenticate
/// 
/// # Arguments
/// * `access_key` - String
/// * `secret_key` - String
/// * `region` - String
pub fn authenticate(access_key: &str, secret_key: &str, region: &str) -> Result<(), Error> {
    write_aws_config(
        access_key.to_owned(),
        secret_key.to_owned(),
        region.to_owned()
    )?;
    test_authentication()?;

    Ok(())
}
