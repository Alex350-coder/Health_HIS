//! Argon2id password hashing (Security.md Section 2). Parameters — memory 19456 KiB,
//! 2 iterations, 1 degree of parallelism — are the OWASP-recommended baseline, fixed here so any
//! future tuning is a reviewed code change rather than an incidental drift.

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Argon2, Params, Version};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HashingError {
    #[error("password hashing failed: {0}")]
    Hash(argon2::password_hash::Error),
}

fn argon2() -> Argon2<'static> {
    // Security.md Section 2 — memory=19456 KiB, iterations=2, parallelism=1. These are fixed,
    // known-valid constants; `Params::default()` (infallible) is only a defensive fallback that
    // can never actually be reached.
    let params = Params::new(19456, 2, 1, None).unwrap_or_else(|_| Params::default());
    Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params)
}

pub fn hash_password(password: &str) -> Result<String, HashingError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2()
        .hash_password(password.as_bytes(), &salt)
        .map_err(HashingError::Hash)?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, HashingError> {
    let parsed_hash = PasswordHash::new(hash).map_err(HashingError::Hash)?;
    Ok(argon2()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_correct_password() {
        let hash = hash_password("Correct-Horse-Battery-Staple-1").expect("hashing succeeds");
        assert!(verify_password("Correct-Horse-Battery-Staple-1", &hash).expect("verify runs"));
    }

    #[test]
    fn rejects_an_incorrect_password() {
        let hash = hash_password("Correct-Horse-Battery-Staple-1").expect("hashing succeeds");
        assert!(!verify_password("wrong-password", &hash).expect("verify runs"));
    }

    #[test]
    fn produces_a_different_hash_each_time_due_to_random_salt() {
        let first = hash_password("Correct-Horse-Battery-Staple-1").expect("hashing succeeds");
        let second = hash_password("Correct-Horse-Battery-Staple-1").expect("hashing succeeds");
        assert_ne!(first, second);
    }

    #[test]
    fn rejects_a_malformed_stored_hash() {
        let result = verify_password("anything", "not-a-valid-hash");
        assert!(result.is_err());
    }
}
