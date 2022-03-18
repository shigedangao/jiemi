use crate::err::Error;

// Constant
const VAULT_TOKEN_VAR: &str = "VAULT_TOKEN";

/// Set the vault token in order to authenticate with vault
/// 
/// # Arguments
/// * `token` - &str
pub(crate) fn set_vault_token(token: &str) -> Result<(), Error> {
    std::env::set_var(VAULT_TOKEN_VAR, token);
    
    Ok(())
}