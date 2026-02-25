use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub enum GoogleClaimsError {
    InvalidTokenId,
    MissingKid,
    FailedToGetKeyFromGoogle,
    InvalidResponseTypeFromGoogle,
    InvalidKeyComponentFromGoogle,
    FailedToDecodeRsaComponents,
    KeyNotFound,
    ExpiredToken,
    FailedToDecodeKeyFromGoogle,
    FailedToSetJwkSetFromGoogle,
    FailedToDecodeHeader,
    FailedToGetHeaderSlice,
    FailedToGetTokenDataClaims,
    FailedToValidateTokenFromGoogle,
    ExpiredOrInvalidToken,
    FailedToDecodeAuthResponseFromGoogle,
    InvalidIssuer,
    MissingIssuer,
    InvalidClientId,
    MissingClientId,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum AuthenticateUserError {
    InvalidToken,
    FailedToInsertUser,
    FailedToGenerateSessionToken,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GoogleTokenInfoResponse {
    pub iss: String,
    pub azp: Option<String>,
    pub aud: String,
    pub sub: String,
    pub email: String,
    pub email_verified: Option<String>,
    pub nbf: Option<String>,
    pub name: String,
    pub picture: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub iat: Option<String>,
    pub exp: String,
    pub jti: Option<String>,
    pub alg: Option<String>,
    pub kid: Option<String>,
    pub typ: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SessionTokenClaims {
    pub user_id: Uuid,
    pub google_sub: String,
    pub email: Option<String>,
    pub exp: usize,
}
