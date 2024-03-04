use std::collections::HashMap;
use argon2::Config;
use getrandom::getrandom;

pub struct PasswordStore {
    master_password: Vec<u8>,
    passwords: HashMap<String, String>,
}

fn derivate_key(
    password: &[u8],
    config: Option<Config>,
) -> anyhow::Result<Vec<u8>> {
    let config = config.unwrap_or(Config::owasp5());
    let mut salt = [0; 16];
    let _ = getrandom(&mut salt);
    Ok(argon2::hash_raw(password, &salt, &config)?)
}

impl PasswordStore {
    pub fn new(password: &[u8]) -> anyhow::Result<Self> {
        let derivated_master_password = derivate_key(password, None)?;
        Ok(Self {
            master_password: derivated_master_password,
            passwords: HashMap::new()
        })
    }
}
