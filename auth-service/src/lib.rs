use base64::{Engine, engine::general_purpose};
use dotenv::dotenv;
use jsonwebtoken::{Algorithm, Validation, jwk::JwkSet};
use reqwest::{Client, StatusCode};
use std::error::Error as StdError;
use utility_helpers::types::{EnvVarConfig, GoogleClaims};
use uuid::Uuid;

use token_services::Claims;
use types::{
    AuthenticateUserError, GoogleClaimsError, GoogleTokenInfoResponse, SessionTokenClaims,
};

pub mod token_services;
pub mod types;

#[derive(Clone)]
pub struct AuthService {
    pub client: Client,
    pub pool: sqlx::PgPool,
    pub env_var_config: EnvVarConfig,
}

impl AuthService {
    pub fn new(pg_pool: sqlx::PgPool) -> Result<Self, Box<dyn StdError>> {
        dotenv().ok();

        let client = Client::new();
        let env_var_config = EnvVarConfig::new()?;

        let auth_service = AuthService {
            client,
            env_var_config,
            pool: pg_pool,
        };

        Ok(auth_service)
    }

    pub fn get_claims(data: String, exp: usize) -> Claims {
        Claims::new(data, exp)
    }

    pub async fn get_google_claims(
        &self,
        id_token: &String,
    ) -> Result<GoogleClaims, GoogleClaimsError> {
        let id_token_component = id_token.split(".").collect::<Vec<_>>();

        if id_token_component.len() != 3 {
            return Err(GoogleClaimsError::InvalidTokenId);
        }

        let client = self.client.clone();

        let header_json = general_purpose::URL_SAFE_NO_PAD
            .decode(id_token_component[0])
            .map_err(|_| GoogleClaimsError::FailedToDecodeHeader)?;
        let header: serde_json::Value = serde_json::from_slice(&header_json)
            .map_err(|_| GoogleClaimsError::FailedToGetHeaderSlice)?;
        let kid = header
            .get("kid")
            .and_then(|k| k.as_str())
            .ok_or(GoogleClaimsError::MissingKid)?;

        let jwks_url = "https://www.googleapis.com/oauth2/v3/certs";
        let auth_url = format!(
            "https://oauth2.googleapis.com/tokeninfo?id_token={}",
            id_token
        );

        let (token_auth_response, jwk_sets_response) = tokio::join!(
            get_token_auth_resp(&client, &auth_url, &self.env_var_config.google_client_id),
            get_jwk_set_resp_string(&client, jwks_url)
        );

        let (token_auth_response, jwk_sets_response) =
            match (token_auth_response, jwk_sets_response) {
                (Ok(token_auth_response), Ok(jwk_sets_response)) => {
                    (token_auth_response, jwk_sets_response)
                }
                (Err(e), _) => return Err(e),
                (_, Err(e)) => return Err(e),
            };

        let jwk_set: JwkSet = serde_json::from_str(&jwk_sets_response)
            .map_err(|_| GoogleClaimsError::FailedToSetJwkSetFromGoogle)?;

        let google_pk_kid = jwk_set
            .keys
            .iter()
            .find(|key| key.common.key_id.as_deref() == Some(kid))
            .ok_or(GoogleClaimsError::KeyNotFound)?;

        if token_auth_response.kid.unwrap() != kid
            || google_pk_kid.common.key_id != Some(kid.to_string())
        {
            return Err(GoogleClaimsError::InvalidTokenId);
        }

        Ok(GoogleClaims {
            sub: token_auth_response.sub,
            email: token_auth_response.email,
            name: token_auth_response.name,
            picture: token_auth_response.picture,
            exp: token_auth_response
                .exp
                .parse::<usize>()
                .map_err(|_| GoogleClaimsError::FailedToDecodeRsaComponents)?,
        })
    }

    fn generate_session_token(
        &self,
        google_claims: &GoogleClaims,
        user_id: Uuid,
    ) -> Result<String, Box<dyn StdError>> {
        let current_time = chrono::Utc::now().timestamp() as usize;
        let session_claims = SessionTokenClaims {
            user_id,
            google_sub: google_claims.sub.clone(),
            email: Some(google_claims.email.clone()),
            exp: current_time + 60 * 60 * 24 * 30, // 30 days
        };

        let session_token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &session_claims,
            &jsonwebtoken::EncodingKey::from_secret(self.env_var_config.jwt_secret.as_ref()),
        )
        .map_err(|_| "Failed to encode session token".to_string())?;

        Ok(session_token)
    }

    pub fn verify_session_token(
        &self,
        session_token: &str,
    ) -> Result<SessionTokenClaims, Box<dyn StdError>> {
        let validation = Validation::new(Algorithm::HS256);

        let token_data = jsonwebtoken::decode::<SessionTokenClaims>(
            session_token,
            &jsonwebtoken::DecodingKey::from_secret(self.env_var_config.jwt_secret.as_ref()),
            &validation,
        )
        .map_err(|e| {
            println!("Error decoding session token: {:?}", e);
            "Failed to decode session token".to_string()
        })?;
        let claims = token_data.claims;
        let current_time = chrono::Utc::now().timestamp() as usize;
        if claims.exp < current_time {
            return Err("Session token expired".into());
        }

        Ok(claims)
    }

    pub async fn authenticate_user(
        &self,
        id_token: &String,
    ) -> Result<(Uuid, String, bool), AuthenticateUserError> {
        // validate google id token
        let google_claims = self
            .get_google_claims(&id_token)
            .await
            .map_err(|_| AuthenticateUserError::InvalidToken)?;

        // check or create user in db
        let (user, is_new_user) = db_service::schema::users::User::create_or_update_existing_user(
            &self.pool,
            &google_claims,
        )
        .await
        .map_err(|_| AuthenticateUserError::FailedToInsertUser)?;

        // generate session token
        let session_token = self
            .generate_session_token(&google_claims, user.id)
            .map_err(|_| AuthenticateUserError::FailedToGenerateSessionToken)?;

        Ok((user.id, session_token, is_new_user))
    }
}

async fn get_token_auth_resp(
    client: &Client,
    url: &str,
    google_client_id: &str,
) -> Result<GoogleTokenInfoResponse, GoogleClaimsError> {
    let req = client
        .get(url)
        .send()
        .await
        .map_err(|_| GoogleClaimsError::FailedToValidateTokenFromGoogle)?;

    if req.status() != StatusCode::OK {
        return Err(GoogleClaimsError::ExpiredOrInvalidToken);
    }

    let response = req
        .json::<GoogleTokenInfoResponse>()
        .await
        .map_err(|_| GoogleClaimsError::FailedToDecodeAuthResponseFromGoogle)?;

    if response.iss != "accounts.google.com" && response.iss != "https://accounts.google.com" {
        return Err(GoogleClaimsError::InvalidIssuer);
    }

    if response.aud != google_client_id {
        return Err(GoogleClaimsError::InvalidClientId);
    }

    if let Some(kid) = &response.kid {
        if kid.is_empty() {
            return Err(GoogleClaimsError::MissingKid);
        }
    } else {
        return Err(GoogleClaimsError::MissingKid);
    }

    Ok(response)
}

async fn get_jwk_set_resp_string(client: &Client, url: &str) -> Result<String, GoogleClaimsError> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|_| GoogleClaimsError::FailedToGetKeyFromGoogle)?
        .text()
        .await
        .map_err(|_| GoogleClaimsError::FailedToDecodeKeyFromGoogle)?;

    Ok(response)
}
