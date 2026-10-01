use argon2::{Argon2, password_hash::PasswordHasher};
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

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use crate::crypto::generate_password_hash;

    #[test]
    fn test_add() {
        let mut rng = StdRng::seed_from_u64(42);

        let password = "password123!".into();

        let (salt, hash) = generate_password_hash(&password, &mut rng).unwrap();

        assert_eq!(salt, "IhPi3oZCnaWvL2oI"); // really just making sure this is deterministic
        assert_eq!(
            hash,
            "$argon2id$v=19$m=19456,t=2,p=1$SWhQaTNvWkNuYVd2TDJvSQ$whersSRXMEV6FQmlVQPSt4XOJc1pKF9ZCd2bKJNJZyY"
        );
    }
}
