use crate::db;
use crate::db::users::User;
use crate::extractors::Jwt;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;
use sqlx::{Error, Pool, Sqlite};
use std::collections::BTreeMap;
use std::env;
use std::sync::Arc;

lazy_static! {
    static ref JWT_SECRET: String = env::var("JWT_SECRET").unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Signup {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Login {
    pub email: String,
    pub password: String,
}

pub async fn create_user(
    Extension(pool): Extension<Arc<Pool<Sqlite>>>,
    Json(signup): Json<Signup>,
) -> impl IntoResponse {
    if !(*crate::features::SIGNUPS_ENABLED) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Signups are not enabled" })),
        );
    }
    // TODO: Potentially more checks for password strength
    if signup.password.len() < 12 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Password must be at least 12 characters"})),
        );
    }
    let user =
        db::users::create_user(&pool, &signup.email, &signup.password, &signup.display_name).await;
    match user {
        Ok(user) => (StatusCode::OK, Json(json!({ "token": issue_jwt(user) }))),
        Err(err) => match err {
            Error::Database(e) if e.code().unwrap_or(std::borrow::Cow::Borrowed("")) == "23505" => {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "A user with that email already exists"})),
                )
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("{:?}", err) })),
            ),
        },
    }
}

pub async fn whoami(Jwt(user): Jwt) -> impl IntoResponse {
    (StatusCode::OK, Json(json!(user)))
}

#[axum::debug_handler]
pub async fn login(
    Extension(pool): Extension<Arc<Pool<Sqlite>>>,
    Json(login_req): Json<Login>,
) -> impl IntoResponse {
    let user = db::users::get_user(&pool, &login_req.email).await;
    let user = match user {
        Ok(user) => user,
        Err(err) => match err {
            Error::RowNotFound => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Invalid email or password"})),
                );
            }
            _ => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("{:?}", err) })),
                )
            }
        },
    };

    if !bcrypt::verify(&login_req.password, &user.password).unwrap_or(false) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid email or password"})),
        );
    }
    if !user.enabled {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid email or password"})),
        );
    }

    let token = issue_jwt(user);
    (StatusCode::OK, Json(json!({ "token": token })))
}

fn issue_jwt(user: User) -> String {
    let key: Hmac<Sha256> = Hmac::new_from_slice((*JWT_SECRET).as_bytes()).unwrap();
    let mut claims = BTreeMap::new();

    let iat = chrono::Utc::now().timestamp().to_string();
    let exp = (chrono::Utc::now() + chrono::Duration::hours(7 * 24))
        .timestamp()
        .to_string();
    let dn = user.display_name.clone();
    let admin = user.admin.to_string();
    let sub = user.id.to_string();

    // https://www.iana.org/assignments/jwt/jwt.xhtml
    claims.insert("iss", "integral");
    claims.insert("sub", &sub);
    claims.insert("iat", &iat);
    claims.insert("exp", &exp);
    claims.insert("dn", &dn);
    claims.insert("email", &user.email);
    claims.insert("admin", &admin);

    claims.sign_with_key(&key).unwrap()
}
