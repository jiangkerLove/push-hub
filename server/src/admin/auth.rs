use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::AppError;
use crate::AppResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    /// Unix 秒，与 admin_users.password_updated_at 一致；密码变更后旧 token 失效
    #[serde(default)]
    pub pwd_at: i64,
}

pub fn hash_password(password: &str) -> AppResult<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Config(format!("password hash failed: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    match bcrypt::verify(password, hash) {
        Ok(valid) => Ok(valid),
        Err(err) => {
            tracing::warn!(error = %err, "password hash verify failed; treating as invalid credentials");
            Ok(false)
        }
    }
}

pub fn create_token(username: &str, password_version: i64, config: &Config) -> AppResult<String> {
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .ok_or_else(|| AppError::Config("invalid token expiry".into()))?
        .timestamp() as usize;

    let claims = Claims {
        sub: username.to_string(),
        exp,
        pwd_at: password_version,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Config(format!("token encode failed: {e}")))
}

pub fn verify_token(token: &str, config: &Config) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("invalid or expired token".into()))
}
