use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash::PasswordHasher};
use rand::{
    Rng,
    distr::{Alphanumeric, SampleString},
};

pub fn generate_password_hash<R: Rng>(
    password: &String,
    rng: &mut R,
) -> Result<(String, String), argon2::password_hash::Error> {
    let salt = Alphanumeric.sample_string(rng, 16);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password_with_salt(password.as_bytes(), salt.as_bytes())?;

    Ok((salt, password_hash.to_string()))
}

pub fn verify_password(password: &str, password_hash: &str) -> crate::error::Result<()> {
    let ph = PasswordHash::new(password_hash).unwrap();

    Argon2::default()
        .verify_password(password.as_bytes(), &ph)
        .map_err(|err| match err {
            argon2::password_hash::Error::PasswordInvalid => {
                crate::error::Error::InvalidCredentials
            }
            _ => crate::error::Error::PasswordHash(err),
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use crate::crypto::{generate_password_hash, verify_password};

    #[test]
    fn test_hash_password() {
        let mut rng = StdRng::seed_from_u64(42);

        let password = "password123!".into();

        let (salt, hash) = generate_password_hash(&password, &mut rng).unwrap();

        assert_eq!(salt, "IhPi3oZCnaWvL2oI"); // really just making sure this is deterministic
        assert_eq!(
            hash,
            "$argon2id$v=19$m=19456,t=2,p=1$SWhQaTNvWkNuYVd2TDJvSQ$whersSRXMEV6FQmlVQPSt4XOJc1pKF9ZCd2bKJNJZyY"
        );
    }

    #[test]
    fn test_verify_password() {
        let password = "password123!".into();
        let hash = "$argon2id$v=19$m=19456,t=2,p=1$SWhQaTNvWkNuYVd2TDJvSQ$whersSRXMEV6FQmlVQPSt4XOJc1pKF9ZCd2bKJNJZyY".into();

        verify_password(password, hash).unwrap();
    }
}
