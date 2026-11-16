use keyring::Entry;

const SERVICE_NAME: &str = "com.novelper.app";

pub fn store_api_key(secret_ref: &str, api_key: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, secret_ref).map_err(|e| e.to_string())?;
    entry.set_password(api_key).map_err(|e| e.to_string())
}

pub fn load_api_key(secret_ref: &str) -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE_NAME, secret_ref).map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(pw) => Ok(Some(pw)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn delete_api_key(secret_ref: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, secret_ref).map_err(|e| e.to_string())?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
