use std::collections::BTreeMap;
use std::env;

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::{header, request::Parts, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::response::Response;
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;

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
pub struct Jwt(pub Token);

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
                let key: Hmac<Sha256> = Hmac::new_from_slice((*JWT_SECRET).as_bytes()).unwrap();
                let token = header.replace("Bearer ", "");
                // if token.starts_with("api_") {
                //     let mut hasher = Sha256::new();
                //     hasher.update(token.as_bytes());
                //     let digest = hasher.finalize();
                //     let hash = hex::encode(digest);

                //     let db_pool = req.extensions().get::<Arc<Pool<Postgres>>>().unwrap();

                //     let user = match crate::db::users::get_user_from_api_key(db_pool, &hash).await {
                //         Ok(user) => user,
                //         Err(err) => {
                //             return Err((
                //                 StatusCode::INTERNAL_SERVER_ERROR,
                //                 Json(json!({"error": err.to_string()})),
                //             ))
                //         }
                //     };
                //     let token = Token {
                //         iss: "api".to_owned(),
                //         sub: user.id,
                //         iat: 0,
                //         exp: 0,
                //         dn: user.email.to_owned(),
                //         email: user.email.to_owned(),
                //         admin: user.admin,
                //     };
                //     return Ok(Self(token));
                // }
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

                return Ok(Self(token));
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
