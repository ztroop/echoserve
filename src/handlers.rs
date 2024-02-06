use axum::response::IntoResponse;
use axum::{http::StatusCode, response::Json};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

// Default handler for no YAML configuration
pub async fn default_handler() -> &'static str {
    "OK"
}

// Handler for custom route
pub async fn handle_custom_route(
    data: Arc<Value>,
    status: StatusCode,
    headers: Arc<HashMap<String, String>>,
) -> impl axum::response::IntoResponse {
    let mut response = axum::response::Response::new(Json(data.as_ref().clone()).into_response());
    *response.status_mut() = status;

    for (key, value) in headers.as_ref() {
        response.headers_mut().insert(
            axum::http::header::HeaderName::from_str(key).unwrap(),
            axum::http::header::HeaderValue::from_str(value).unwrap(),
        );
    }

    response
}
