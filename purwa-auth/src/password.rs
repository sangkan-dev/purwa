//! Argon2id password hashing (PHC string format).
//!
//! Passwords are **hashed**, not encrypted: there is no decryption step, only verification.
//! Defaults follow OWASP-style guidance: prefer **Argon2id** with sufficient memory cost.
//! Tune `m_cost_kib`, `t_cost`, and `p_cost` for your environment (higher is slower but harder
//! to brute-force). For **tests** or very slow CI, use [`hash_password_fast`] / [`ARGON2_FAST`].

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use rand_core::OsRng;

use crate::PasswordError;

/// Memory cost in **kibibytes** (KiB). OWASP suggests large values for production; ~19 MiB is a
/// common baseline (`19 * 1024` KiB).
pub const DEFAULT_M_COST_KIB: u32 = 19 * 1024;
/// Time cost (iterations).
pub const DEFAULT_T_COST: u32 = 2;
/// Parallelism lanes.
pub const DEFAULT_P_COST: u32 = 1;

/// Low-cost parameters for unit tests and local dev only (not for production secrets).
pub const ARGON2_FAST: Params = match Params::new(256, 2, 1, None) {
    Ok(p) => p,
    Err(_) => panic!("argon2 fast params"),
};

/// Production-oriented Argon2id hasher.
pub fn argon2_default() -> Argon2<'static> {
    let params = Params::new(DEFAULT_M_COST_KIB, DEFAULT_T_COST, DEFAULT_P_COST, None)
        .expect("valid argon2 params");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

fn argon2_with_params(params: Params) -> Argon2<'static> {
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

/// Hash a plaintext password to a PHC string (`$argon2id$...`) using default costs.
pub fn hash_password(plain: &str) -> Result<String, PasswordError> {
    hash_password_with(plain, argon2_default())
}

/// Hash using explicit Argon2 instance (e.g. [`argon2_fast`] in tests).
pub fn hash_password_with(plain: &str, argon2: Argon2<'_>) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let phc = argon2
        .hash_password(plain.as_bytes(), &salt)
        .map_err(|e| PasswordError::Hash(e.to_string()))?
        .to_string();
    Ok(phc)
}

/// Low-cost hasher for tests.
pub fn argon2_fast() -> Argon2<'static> {
    argon2_with_params(ARGON2_FAST)
}

/// Hash with [`argon2_fast`] (tests / CI only).
pub fn hash_password_fast(plain: &str) -> Result<String, PasswordError> {
    hash_password_with(plain, argon2_fast())
}

/// Verify `plain` against a stored PHC string.
pub fn verify_password(plain: &str, phc: &str) -> Result<bool, PasswordError> {
    verify_password_with(plain, phc, argon2_default())
}

/// Verify using the same Argon2 **base** as used for hashing; verification reads parameters from
/// the PHC string, so this mainly selects algorithm/version.
pub fn verify_password_with(
    plain: &str,
    phc: &str,
    argon2: Argon2<'_>,
) -> Result<bool, PasswordError> {
    let parsed = PasswordHash::new(phc).map_err(|_| PasswordError::InvalidHash)?;
    Ok(argon2.verify_password(plain.as_bytes(), &parsed).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_fast() {
        let h = hash_password_fast("correct horse battery staple").unwrap();
        assert!(h.starts_with("$argon2id$"));
        assert!(verify_password_with("correct horse battery staple", &h, argon2_fast()).unwrap());
        assert!(!verify_password_with("wrong", &h, argon2_fast()).unwrap());
    }

    #[test]
    fn wrong_password_rejected() {
        let h = hash_password_fast("secret").unwrap();
        assert!(!verify_password_with("Secret", &h, argon2_fast()).unwrap());
    }
}
