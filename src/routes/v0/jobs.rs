use std::{collections::HashMap, sync::Arc};

use axum::{extract::Query, http::StatusCode, response::IntoResponse, Extension, Json};
use serde::Deserialize;
use serde_json::json;
use sqlx::{Pool, Sqlite};

use crate::{db, extractors::Jwt};

pub async fn get_all_jobs(
    Extension(pool): Extension<Arc<Pool<Sqlite>>>,
    Query(params): Query<HashMap<String, String>>,
    Jwt(_user): Jwt,
) -> impl IntoResponse {
    if let Some(id) = params.get("id") {
        let job = db::jobs::get_job_by_id(&pool, &id).await;
        match job {
            Ok(job) => (StatusCode::OK, Json(json!(job))),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(e.to_string())),
            ),
        }
    } else {
        let jobs = db::jobs::get_all_jobs(&pool).await;
        match jobs {
            Ok(jobs) => (StatusCode::OK, Json(json!(jobs))),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(e.to_string())),
            ),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateJob {
    pub synopsis: String,
    pub location: Option<String>,
}

pub async fn create_job(
    Extension(pool): Extension<Arc<Pool<Sqlite>>>,
    Jwt(user): Jwt,
    Json(data): Json<CreateJob>,
) -> impl IntoResponse {
    let created_job = db::jobs::create_job(&pool, &data.synopsis, data.location, &user.sub).await;
    match created_job {
        Ok(job) => (StatusCode::OK, Json(json!(job))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(e.to_string())),
        ),
    }
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateComment {
    pub job_id: String,
    pub comment: String,
}

pub async fn add_comment(
    Extension(pool): Extension<Arc<Pool<Sqlite>>>,
    Jwt(user): Jwt,
    Json(data): Json<CreateComment>,
) -> impl IntoResponse {
    let created_comment = db::jobs::add_comment(&pool, &data.job_id, &data.comment, &user.sub).await;
    match created_comment {
        Ok(c) => (StatusCode::OK, Json(json!(c))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(e.to_string())),
        ),
    }
}