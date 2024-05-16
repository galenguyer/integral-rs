use std::{collections::HashMap, sync::Arc};

use axum::{extract::Query, http::StatusCode, response::IntoResponse, Extension, Json};
use serde::Deserialize;
use serde_json::json;
use sqlx::{Pool, Sqlite};

use crate::{db, extractors::Jwt};

pub async fn get_all_resources(
    Extension(pool): Extension<Arc<Pool<Sqlite>>>,
    Jwt(_user): Jwt,
) -> impl IntoResponse {
    let resources = db::resources::list(&pool).await;
    match resources {
        Ok(job) => (StatusCode::OK, Json(json!(job))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(e.to_string())),
        ),
    }
}
