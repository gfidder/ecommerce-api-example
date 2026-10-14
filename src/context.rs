use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

use crate::{
    config::{Config, RsaJwtConfig},
    error::Report,
    models::{TokenClaims, TokenDetails},
};

#[derive(Clone)]
pub struct AuthContext {
    pub access: JwtContext,
    pub refresh: JwtContext,
}

#[derive(Clone)]
pub struct JwtContext {
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
    pub exp: i64,
}

impl JwtContext {
    pub fn generate_token(&self, sub: Uuid) -> Result<TokenDetails, Report> {
        let now = chrono::Utc::now();

        let mut token_details = TokenDetails {
            user_pid: sub,
            token_id: Uuid::new_v4(),
            expires_in: Some((now + chrono::Duration::seconds(self.exp)).timestamp()),
            token: None,
        };

        let claims = TokenClaims {
            sub: token_details.user_pid.to_string(),
            id: token_details.token_id.to_string(),
            exp: token_details.expires_in.unwrap_or(0),
            iat: now.timestamp(),
            nbf: now.timestamp(),
        };

        let header = Header::default();

        let token = jsonwebtoken::encode(&header, &claims, &self.encoding_key)?;

        token_details.token = Some(token);

        Ok(token_details)
    }

    pub fn verify_token(&self, token: &str) -> Result<TokenDetails, Report> {
        let validation = Validation::default();

        let token_data =
            jsonwebtoken::decode::<TokenClaims>(token, &self.decoding_key, &validation)?;

        let user_pid = Uuid::parse_str(&token_data.claims.sub).unwrap();
        let token_id = Uuid::parse_str(&token_data.claims.id).unwrap();

        Ok(TokenDetails {
            token: None,
            token_id,
            user_pid,
            expires_in: None,
        })
    }
}

impl TryFrom<&RsaJwtConfig> for JwtContext {
    type Error = Report;

    fn try_from(config: &RsaJwtConfig) -> Result<Self, Self::Error> {
        let encoding_key = config.encoding_key()?;
        let decoding_key = config.decoding_key()?;

        let exp = config.exp();

        Ok(Self {
            encoding_key,
            decoding_key,
            exp,
        })
    }
}
