use argon2::{
    Argon2, Params,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use std::env;

use dotenv::dotenv;

pub fn hash_pass(word: &str) -> Result<String, argon2::password_hash::Error> {
    dotenv().ok();
    let pepper = env::var("PEPPER")
        .expect("PEPPER Should be set to ensure password security.")
        .into_bytes();

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::new_with_secret(
        &pepper,
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::default(),
    )?;

    let hash = argon2.hash_password(word.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub fn verify_hash(stored_hash: &str, password: &str) -> bool {
    dotenv().ok();
    let pepper = env::var("PEPPER")
        .expect("PEPPER var must be set.")
        .into_bytes();

    let parse = match PasswordHash::new(stored_hash) {
        Ok(hash) => hash,
        Err(e) => return false,
    };

    Argon2::new_with_secret(
        &pepper,
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::default(),
    )
    .expect("Failed to ini Argon2")
    .verify_password(password.as_bytes(), &parse)
    .is_ok()
}
