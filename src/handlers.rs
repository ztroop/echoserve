use crate::structs::SequenceResponse;
use axum::response::IntoResponse;
use axum::{http::StatusCode, response::Json};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

// Default handler for no YAML configuration
pub async fn default_handler() -> StatusCode {
    StatusCode::OK
}

// Handler for custom route
pub async fn handle_custom_route(
    data: Arc<Option<Value>>,
    status: StatusCode,
    headers: Arc<HashMap<String, String>>,
) -> impl IntoResponse {
    let response_body = match data.as_ref() {
        None => status.into_response(),
        Some(payload) => match payload {
            Value::String(actual_value) => (status, actual_value.clone()).into_response(),
            _ => Json(payload.clone()).into_response(),
        },
    };

    let mut response = response_body;
    *response.status_mut() = status;
    for (key, value) in headers.as_ref() {
        response.headers_mut().insert(
            axum::http::header::HeaderName::from_str(key).unwrap(),
            axum::http::header::HeaderValue::from_str(value).unwrap(),
        );
    }

    response
}

pub async fn handle_sequence_route(
    endpoint: String,
    sequence: Vec<SequenceResponse>,
    counters: Arc<Mutex<HashMap<String, usize>>>,
) -> impl IntoResponse {
    let mut counters = counters.lock().await;
    let count = counters.entry(endpoint.clone()).or_insert(0);
    let idx = *count;
    *count += 1;
    drop(counters);

    let resp = if idx < sequence.len() {
        &sequence[idx]
    } else {
        sequence.last().unwrap()
    };

    let data = resp.data.clone().unwrap_or_default();
    let mut headers = resp.headers.clone().unwrap_or_default();
    headers.insert(
        "content-type".to_string(),
        data.format.as_content_type().to_string(),
    );
    let route_payload = match data.payload {
        Some(crate::structs::JsonOrString::Json(value)) => Some(value),
        Some(crate::structs::JsonOrString::Str(value)) => Some(serde_json::json!(value)),
        None => None,
    };
    let status_code = resp
        .status
        .map(|s| StatusCode::from_u16(s).unwrap_or(StatusCode::OK))
        .unwrap_or(StatusCode::OK);
    handle_custom_route(Arc::new(route_payload), status_code, Arc::new(headers)).await
}
