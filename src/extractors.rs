use std::collections::BTreeMap;
use std::env;
use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::{header, request::Parts, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::response::Response;
use axum::{async_trait, Extension, RequestPartsExt};
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;
use sqlx::{Pool, Sqlite};

use crate::db;
use crate::db::users::User;

lazy_static! {
    static ref JWT_SECRET: String = env::var("JWT_SECRET").unwrap();
}

pub struct Json<T>(pub T);

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub iss: String,
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub dn: String,
    pub email: String,
    pub admin: bool,
}
pub struct Jwt(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for Jwt
where
    S: Send + Sync,
{
    type Rejection = (axum::http::StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Grab the "Authorization" header from the request
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok());

        match auth_header {
            Some(header) => {
                let token = header.replace("Bearer ", "");
                let Extension(db_pool) = parts
                    .extract::<Extension<Arc<Pool<Sqlite>>>>()
                    .await
                    .map_err(|err| err.into_response())
                    .unwrap();

                let user = get_user_from_token(&db_pool, &token).await;

                match user {
                    Ok(user) => Ok(Self(user)),
                    Err(e) => Err(e),
                }
            }
            None => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "missing auth header"})),
                ))
            }
        }
    }
}

pub async fn get_user_from_token(
    pool: &Pool<Sqlite>,
    token: &str,
) -> Result<User, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let key: Hmac<Sha256> = Hmac::new_from_slice((*JWT_SECRET).as_bytes()).unwrap();

    let claims: BTreeMap<String, String> = match token.verify_with_key(&key) {
        Ok(claims) => claims,
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid token" })),
            ))
        }
    };

    let token = Token {
        iss: claims.get("iss").unwrap().to_string(),
        sub: claims.get("sub").unwrap().to_string(),
        iat: claims.get("iat").unwrap().parse().unwrap(),
        exp: claims.get("exp").unwrap().parse().unwrap(),
        dn: claims.get("dn").unwrap().to_string(),
        email: claims.get("email").unwrap().to_string(),
        admin: claims.get("admin").unwrap().parse().unwrap(),
    };

    let now = chrono::Utc::now().timestamp();
    if token.iat > now || token.exp < now {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid token"})),
        ));
    }

    match db::users::get_user(&pool, &token.sub).await {
        Ok(user) => Ok(user),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "error fetching user from db", "details": e.to_string()})),
        )),
    }
}

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match serde_json::to_vec(&self.0) {
            Ok(bytes) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                )],
                bytes,
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}
