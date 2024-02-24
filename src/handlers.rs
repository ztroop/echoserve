use axum::response::IntoResponse;
use axum::{http::StatusCode, response::Json};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

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
