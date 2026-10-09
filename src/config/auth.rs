use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::Deserialize;
use std::path::PathBuf;

use crate::error::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct RsaJwtConfig {
    secret: String,
    exp: i64,
}

impl RsaJwtConfig {
    pub fn encoding_key(&self) -> Result<EncodingKey> {
        Ok(EncodingKey::from_secret(self.secret.as_bytes()))
    }

    pub fn decoding_key(&self) -> Result<DecodingKey> {
        Ok(DecodingKey::from_secret(self.secret.as_bytes()))
    }

    pub fn exp(&self) -> i64 {
        self.exp
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    access: RsaJwtConfig,
    refresh: RsaJwtConfig,
}

impl AuthConfig {
    pub fn access(&self) -> &RsaJwtConfig {
        &self.access
    }

    pub fn refresh(&self) -> &RsaJwtConfig {
        &self.refresh
    }
}
