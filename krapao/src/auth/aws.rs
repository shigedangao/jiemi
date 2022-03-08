use std::fs;
use serde::Serialize;
use crate::err::Error;
use crate::helper;

// Constant
const AWS_CONFIG_FOLDER: &str = ".aws";
const AWS_CREDENTIALS_PATH: &str = "credentials";
const AWS_REGION_PATH: &str = "config";
const AWS_OUTPUT: &str = "json";

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

impl AwsConfig {
    /// Create a new AwsConfig based on the input parameter
    /// 
    /// # Arguments
    /// * `creds` - Option<(String, String)>
    /// * `conf` - Option<String>
    fn new(creds: Option<(String, String)>, conf: Option<String>) -> AwsConfig {
        let mut config = AwsConfig::default();
        if let Some((key, access_key)) = creds {
            config.credentials = Some(AwsCredentials {
                aws_access_key_id: key,
                aws_secret_access_key: access_key
            });
        }

        if let Some(region) = conf {
            config.config = Some(AwsRegion {
                region,
                output: AWS_OUTPUT.to_owned()
            });
        }

        config
    }

    /// Create a set of aws credentials which is gonna be stored in ~/.aws/credentials
    /// 
    /// # Arguments
    /// * `&self` - &Self
    fn create_credentials(&self) -> Result<(), Error> {
        let mut aws_path = dirs::home_dir().unwrap_or_default();
        aws_path.push(AWS_CONFIG_FOLDER);
        // create the path if it does not exist
        helper::create_path(&aws_path)
            .map_err(|err| Error::ProviderAuth(err.to_string()))?;

        let mut credentials_path = aws_path.clone();
        credentials_path.push(AWS_CREDENTIALS_PATH);
        
        let toml = toml::to_string(self)
            .map_err(|err| Error::ProviderAuth(err.to_string()))?;

        let toml = toml.replace('"', "");

        fs::write(credentials_path, toml)?;

        Ok(())
    }

    /// Create a set of config file which is gonna be stored in ~/.aws/config
    /// 
    /// # Arguments
    /// * `&self` - &Self
    fn create_config(&self) -> Result<(), Error> {
        let mut aws_path = dirs::home_dir().unwrap_or_default();
        aws_path.push(AWS_CONFIG_FOLDER);
        aws_path.push(AWS_REGION_PATH);
        
        let toml = toml::to_string(self)
            .map_err(|err| Error::ProviderAuth(err.to_string()))?;
            
        let toml = toml.replace('"', "");

        fs::write(aws_path, toml)?;

        Ok(())
    }
}

/// Authenticate
/// 
/// # Arguments
/// * `access_key` - String
/// * `secret_key` - String
/// * `region` - String
pub fn authenticate(access_key: &str, secret_key: &str, region: &str) -> Result<(), Error> {
    if access_key.is_empty() || secret_key.is_empty() || region.is_empty() {
        return Err(Error::Sops("AWS configuration missing either the key_id, access_key or the region".to_owned()));
    }

    // Create credentials
    AwsConfig::new(
        Some((access_key.to_owned(), secret_key.to_owned())), 
        None
    ).create_credentials()?;
    
    // Create config
    AwsConfig::new(None, Some(region.to_owned()))
        .create_config()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::authenticate;

    #[test]
    fn expect_to_authenticate_with_aws() {
        let res = authenticate("foo", "bar", "eu-west-3");
        assert!(res.is_ok());
    }

    #[test]
    fn expect_to_return_err() {
        let res = authenticate("", "", "");
        assert!(res.is_err());
    }
}