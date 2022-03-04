use std::fs;
use serde::Serialize;
use crate::err::Error;
use crate::helper;

// Constant
const AWS_CONFIG_FOLDER: &str = ".aws";
const AWS_CREDENTIALS_PATH: &str = "credentials";
const AWS_REGION_PATH: &str = "config";

#[derive(Serialize, Default)]
struct AwsConfig {
    #[serde(rename(serialize = "default"))]
    credentials: Option<AwsCredentials>,
    #[serde(rename(serialize = "default"))]
    config: Option<AwsRegion>
}

#[derive(Serialize)]
struct AwsCredentials {
    aws_access_key_id: String,
    aws_secret_access_key: String
}

#[derive(Serialize)]
struct AwsRegion {
    region: String,
    output: String
}

/// Write credentials file (~/.aws/credentials)
/// 
/// * `access_key` - String
/// * `secret_key` - String
fn write_aws_credentials_file(access_key: String, secret_key: String) -> Result<(), Error> {
    let mut aws_path = dirs::home_dir()
        .ok_or_else(|| Error::Io("Home dir could not be founded".to_owned()))?;

    aws_path.push(AWS_CONFIG_FOLDER);
    aws_path.push(AWS_CREDENTIALS_PATH);

    // create the path if it does not exist
    helper::create_path(&aws_path)?;

    let profile = AwsCredentials {
        aws_access_key_id: access_key,
        aws_secret_access_key: secret_key
    };

    let toml = toml::to_string(&AwsConfig {
        credentials: Some(profile),
        ..Default::default()
    })?;
    let cleaned_toml = clean_generated_toml_string(&toml);

    fs::write(aws_path, cleaned_toml)?;

    Ok(())
}

/// Write Aws Config
/// 
/// # Arguments
/// * `region` - String
fn write_aws_config(region: String) -> Result<(), Error> {
    let mut aws_path = dirs::home_dir()
        .ok_or_else(|| Error::Io("Home dir could not be founded".to_owned()))?;
    aws_path.push(AWS_CONFIG_FOLDER);
    aws_path.push(AWS_REGION_PATH);

    let reg = AwsRegion {
        region,
        output: "json".to_owned()
    };

    let toml = toml::to_string(&AwsConfig {
        config: Some(reg),
        ..Default::default()
    })?;
    let cleaned_toml = clean_generated_toml_string(&toml);

    fs::write(aws_path, &cleaned_toml)?;

    Ok(())
}

/// Authenticate
/// 
/// # Arguments
/// * `access_key` - String
/// * `secret_key` - String
/// * `region` - String
pub fn authenticate(access_key: &str, secret_key: &str, region: &str) -> Result<(), Error> {
    write_aws_credentials_file(access_key.to_owned(), secret_key.to_owned())?;
    write_aws_config(region.to_owned())?;

    Ok(())
}

/// AWS config does not like string quote. Thus we're removing the quote from the generated toml file
/// 
/// # Arguments
/// * `value` - &str
fn clean_generated_toml_string(value: &str) -> String {
    value.replace('"', "")
}