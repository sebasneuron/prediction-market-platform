use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

impl Claims {
    pub fn new(sub: String, exp: usize) -> Self {
        Claims { sub, exp }
    }

    pub fn new_token(&self) -> Result<String, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();

        let header = Header::new(Algorithm::HS256);
        let secret = std::env::var("JWT_SECRET")?;
        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let token = encode(&header, self, &encoding_key)?;

        Ok(token)
    }

    pub fn verify_token(token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();

        let secret = std::env::var("JWT_SECRET")?;
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(token, &decoding_key, &validation)?;

        Ok(token_data.claims)
    }
}
