use std::{collections::HashMap, sync::Arc};

use axum::http::StatusCode;
use axum::response::sse::{Event as SseEvent, Sse};
use axum::response::Response;
use axum::{extract::Query, response::IntoResponse, Extension};
use serde::Serialize;
use serde_json::json;
use tokio::sync::broadcast;

use crate::extractors::{get_user_from_token, Json, Jwt};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Event {
    Job(()),
    Comment(i64),
}

pub async fn stream(
    Query(params): Query<HashMap<String, String>>,
    Extension(pool): Extension<Arc<sqlx::Pool<sqlx::Sqlite>>>,
    Extension(event_tx): Extension<Arc<broadcast::Sender<Event>>>,
) -> Response {
    let user = match params.get("token") {
        Some(token) => {
            let user = get_user_from_token(&pool, token).await;
            match user {
                Ok(u) => u,
                Err(e) => return e.into_response(),
            }
        }
        None => return (StatusCode::UNAUTHORIZED, Json(json!("missing token"))).into_response(),
    };

    struct Guard {
        user_id: String,
    }

    impl Drop for Guard {
        fn drop(&mut self) {}
    }

    tracing::info!("stream opened");

    let stream = async_stream::stream! {
        let _guard = Guard {
            user_id: user.id.clone()
        };
        let mut rx = event_tx.subscribe();
        while let Ok(event) = rx.recv().await {
            yield SseEvent::default().json_data(event);
        }
    };

    Sse::new(stream).into_response()
}
