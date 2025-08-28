use std::fs;
use std::io::Write;
use std::path::PathBuf;

use argon2::password_hash::{PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use color_eyre::{Result, eyre::eyre};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Credentials {
    username: String,
    hash: String,
}

fn credentials_path() -> PathBuf {
    paths::data_dir().join("auth.json")
}

pub fn exists() -> bool {
    credentials_path().exists()
}

pub fn create_admin(username: &str, password: &str) -> Result<()> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| eyre!("failed to hash password: {e}"))?
        .to_string();

    let creds = Credentials {
        username: username.to_string(),
        hash,
    };

    let path = credentials_path();
    fs::create_dir_all(path.parent().unwrap())?;
    let mut file = fs::File::create(&path)?;
    let data = serde_json::to_vec_pretty(&creds)?;
    file.write_all(&data)?;
    Ok(())
}

pub fn verify(username: &str, password: &str) -> Result<bool> {
    let path = credentials_path();
    let data = fs::read_to_string(&path)?;
    let creds: Credentials = serde_json::from_str(&data)?;
    if creds.username != username {
        return Ok(false);
    }
    let parsed = PasswordHash::new(&creds.hash).map_err(|e| eyre!("invalid stored hash: {e}"))?;
    let ok = Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok();
    Ok(ok)
}
