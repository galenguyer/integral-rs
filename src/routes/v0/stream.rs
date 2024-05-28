use std::{collections::HashMap, sync::Arc};

use axum::http::StatusCode;
use axum::response::sse::{Event as SseEvent, Sse};
use axum::response::Response;
use axum::{extract::Query, response::IntoResponse, Extension};
use serde::Serialize;
use serde_json::json;
use tokio::sync::broadcast;

use crate::extractors::{get_user_from_token, Json};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Event {
    Job(String),
    Resource(String),
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
        fn drop(&mut self) {
            tracing::debug!(user = self.user_id, "closed stream")
        }
    }

    let stream = async_stream::stream! {
        let guard = Guard {
            user_id: user.id.clone()
        };
        tracing::debug!(user=guard.user_id, "opened stream");
        let mut rx = event_tx.subscribe();
        while let Ok(event) = rx.recv().await {
            tracing::debug!(user=guard.user_id, "sent event");
            yield SseEvent::default().json_data(event);
        }
    };

    Sse::new(stream).into_response()
}
