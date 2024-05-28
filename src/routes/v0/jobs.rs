use std::{collections::HashMap, sync::Arc};

use axum::{extract::Query, http::StatusCode, response::IntoResponse, Extension, Json};
use serde::Deserialize;
use serde_json::json;
use sqlx::{Pool, Sqlite};
use tokio::sync::broadcast;

use crate::{db, extractors::Jwt};

use super::stream::Event;

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
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateJob {
    pub synopsis: String,
    pub location: Option<String>,
    pub caller_name: Option<String>,
    pub caller_phone: Option<String>,
    pub comments: Option<Vec<String>>,
}

pub async fn create_job(
    Extension(pool): Extension<Arc<Pool<Sqlite>>>,
    Extension(event_tx): Extension<Arc<broadcast::Sender<Event>>>,
    Jwt(user): Jwt,
    Json(data): Json<CreateJob>,
) -> impl IntoResponse {
    let created_job = db::jobs::create_job(
        &pool,
        &data.synopsis,
        data.location,
        data.caller_name,
        data.caller_phone,
        &user.id,
    )
    .await;
    if let Err(e) = created_job {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(e.to_string())),
        );
    }
    let created_job = created_job.unwrap();

    if let Some(comments) = data.comments {
        for comment in comments {
            if let Err(e) = db::jobs::add_comment(&pool, &created_job.id, &comment, &user.id).await
            {
                tracing::error!("{:?}", e);
            }
        }
    }

    let job = db::jobs::get_job_by_id(&pool, &created_job.id).await;

    match event_tx.send(Event::Job(())) {
        Ok(rcount) => {
            tracing::debug!(
                id = created_job.id,
                recievers = rcount - 1,
                "sent job event"
            );
        }
        Err(e) => tracing::error!(
            id = created_job.id,
            error = format!("{}", e),
            "error sending job event"
        ),
    }

    match job {
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
    let created_comment = db::jobs::add_comment(&pool, &data.job_id, &data.comment, &user.id).await;
    match created_comment {
        Ok(c) => (StatusCode::OK, Json(json!(c))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(e.to_string())),
        ),
    }
}

pub async fn close_job(
    Extension(pool): Extension<Arc<Pool<Sqlite>>>,
    Extension(event_tx): Extension<Arc<broadcast::Sender<Event>>>,
    Jwt(user): Jwt,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(id) = params.get("id") {
        let created_comment = db::jobs::close_job(&pool, &id, &user.id).await;

        match event_tx.send(Event::Job(())) {
            Ok(rcount) => {
                tracing::debug!(id = id.clone(), recievers = rcount - 1, "sent job event");
            }
            Err(e) => tracing::error!(
                id = id.clone(),
                error = format!("{}", e),
                "error sending job event"
            ),
        }

        match created_comment {
            Ok(c) => (StatusCode::OK, Json(json!(c))),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(e.to_string())),
            ),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "missing job id"})),
        )
    }
}
