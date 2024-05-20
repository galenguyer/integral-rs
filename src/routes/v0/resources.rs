use std::sync::Arc;

use axum::{
    http::StatusCode, response::IntoResponse, Extension, Json,
};
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResourceCreationRequest {
    display_name: String,
    comment: Option<String>,
}
pub async fn create(
    Extension(pool): Extension<Arc<Pool<Sqlite>>>,
    Jwt(_user): Jwt,
    Json(req): Json<ResourceCreationRequest>,
) -> impl IntoResponse {
    let resource = db::resources::create_resource(&pool, &req.display_name, req.comment).await;
    match resource {
        Ok(res) => (StatusCode::OK, Json(json!(res))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(e.to_string())),
        ),
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetResourceInServiceRequest {
    id: String,
    in_service: bool,
}
pub async fn set_in_service(
    Extension(pool): Extension<Arc<Pool<Sqlite>>>,
    Jwt(_user): Jwt,
    Json(req): Json<SetResourceInServiceRequest>,
) -> impl IntoResponse {
    let resource = db::resources::set_in_service(&pool, &req.id, req.in_service).await;
    match resource {
        Ok(res) => (StatusCode::OK, Json(json!(res))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(e.to_string())),
        ),
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssignmentRequest {
    job_id: String,
    resource_id: String,
}
pub async fn assign(
    Extension(pool): Extension<Arc<Pool<Sqlite>>>,
    Jwt(user): Jwt,
    Json(req): Json<AssignmentRequest>,
) -> impl IntoResponse {
    let resources = db::resources::list(&pool).await;
    match resources {
        Ok(resources) => {
            let resource = resources.iter().find(|r| r.id == req.resource_id);
            match resource {
                Some(resource) => {
                    if resource.assignment.is_some() {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(json!({"error": "that resource is already assigned to a job"})),
                        );
                    }
                    if !resource.in_service {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(json!({"error": "that resource is out of service"})),
                        );
                    }
                }
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"error": "that resource does not exist"})),
                    );
                }
            }
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(e.to_string())),
            );
        }
    }

    let assignment =
        crate::db::assignments::assign(&pool, &req.job_id, &req.resource_id, &user.id).await;
    match assignment {
        Ok(job) => (StatusCode::OK, Json(json!(job))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(e.to_string())),
        ),
    }
}
