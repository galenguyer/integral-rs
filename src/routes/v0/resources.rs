use std::{collections::HashMap, sync::Arc};

use axum::{extract::Query, http::StatusCode, response::IntoResponse, Extension, Json};
use serde::Deserialize;
use serde_json::json;
use sqlx::{Pool, Sqlite};
use tokio::sync::broadcast;

use crate::{db, extractors::Jwt};

use super::stream::Event;

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
    Extension(event_tx): Extension<Arc<broadcast::Sender<Event>>>,
    Jwt(_user): Jwt,
    Json(req): Json<ResourceCreationRequest>,
) -> impl IntoResponse {
    let resource = db::resources::create_resource(&pool, &req.display_name, req.comment).await;
    event_tx.send(Event::Resource(req.display_name)).ok();
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
    Extension(event_tx): Extension<Arc<broadcast::Sender<Event>>>,
    Jwt(user): Jwt,
    Json(req): Json<SetResourceInServiceRequest>,
) -> impl IntoResponse {
    let resource = db::resources::set_in_service(&pool, &req.id, req.in_service, &user.id).await;
    event_tx.send(Event::Resource(req.id)).ok();
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
    Extension(event_tx): Extension<Arc<broadcast::Sender<Event>>>,
    Jwt(user): Jwt,
    Json(req): Json<AssignmentRequest>,
) -> impl IntoResponse {
    let resources = db::resources::list(&pool).await;
    match resources {
        Ok(resources) => {
            let resource = resources.iter().find(|r| r.id == req.resource_id);
            match resource {
                Some(resource) => {
                    if resource.current_assignment.is_some() {
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

    event_tx.send(Event::Resource(req.resource_id)).ok();
    event_tx.send(Event::Job(req.job_id)).ok();

    match assignment {
        Ok(job) => (StatusCode::OK, Json(json!(job))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(e.to_string())),
        ),
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UnAssignmentRequest {
    assignment_id: String,
}
pub async fn unassign(
    Extension(pool): Extension<Arc<Pool<Sqlite>>>,
    Extension(event_tx): Extension<Arc<broadcast::Sender<Event>>>,
    Jwt(user): Jwt,
    Json(req): Json<UnAssignmentRequest>,
) -> impl IntoResponse {
    let assignment = crate::db::assignments::unassign(&pool, &req.assignment_id, &user.id).await;

    event_tx
        .send(Event::Resource(req.assignment_id.clone()))
        .ok();
    event_tx.send(Event::Job(req.assignment_id)).ok();

    match assignment {
        Ok(job) => (StatusCode::OK, Json(json!(job))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(e.to_string())),
        ),
    }
}

pub async fn get_assignments_for_job(
    Extension(pool): Extension<Arc<Pool<Sqlite>>>,
    Query(params): Query<HashMap<String, String>>,
    Jwt(_user): Jwt,
) -> impl IntoResponse {
    if let Some(id) = params.get("id") {
        let assignments = db::assignments::get_assignments_for_job(&pool, &id).await;
        match assignments {
            Ok(assigns) => (StatusCode::OK, Json(json!(assigns))),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(e.to_string())),
            ),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "missing id"})),
        )
    }
}


// TODO: Auth
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetResourceLocationRequest {
    id: String,
    lat: String,
    lon: String,
}
pub async fn set_resource_location(
    Extension(pool): Extension<Arc<Pool<Sqlite>>>,
    Extension(event_tx): Extension<Arc<broadcast::Sender<Event>>>,
    Json(req): Json<SetResourceLocationRequest>,
) -> impl IntoResponse {
    tracing::info!("received new location for resource {}", req.id);
    let resource = db::resources::set_location(&pool, &req.id, &req.lat, &req.lon).await;
    event_tx.send(Event::Resource(req.id)).ok();
    match resource {
        Ok(res) => (StatusCode::OK, Json(json!(res))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(e.to_string())),
        ),
    }
}
