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
        if let (Ok(name), Ok(val)) = (
            axum::http::header::HeaderName::from_str(key),
            axum::http::header::HeaderValue::from_str(value),
        ) {
            response.headers_mut().insert(name, val);
        }
    }

    response
}

pub async fn handle_sequence_route(
    endpoint: String,
    sequence: Vec<SequenceResponse>,
    counters: Arc<Mutex<HashMap<String, usize>>>,
) -> impl IntoResponse {
    let mut counters = counters.lock().await;
    let count = counters.entry(endpoint).or_insert(0);
    let idx = *count;
    // Saturates at usize::MAX; after saturation, keeps returning last response
    *count = count.saturating_add(1);
    drop(counters);

    let default_resp = SequenceResponse::default();
    let resp = if sequence.is_empty() {
        &default_resp
    } else if idx < sequence.len() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;

    async fn response_status_and_body(
        response: axum::http::Response<Body>,
    ) -> (StatusCode, Vec<u8>) {
        let status = response.status();
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body collection failed")
            .to_bytes();
        (status, body.to_vec())
    }

    #[tokio::test]
    async fn default_handler_returns_200() {
        let response = default_handler().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn handle_custom_route_none_payload() {
        let response =
            handle_custom_route(Arc::new(None), StatusCode::OK, Arc::new(HashMap::new()))
                .await
                .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn handle_custom_route_string_payload() {
        let response = handle_custom_route(
            Arc::new(Some(Value::String("hello".to_string()))),
            StatusCode::OK,
            Arc::new(HashMap::new()),
        )
        .await
        .into_response();
        let (status, body) = response_status_and_body(response).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, b"hello");
    }

    #[tokio::test]
    async fn handle_custom_route_json_payload() {
        let payload = serde_json::json!({"message": "test"});
        let response = handle_custom_route(
            Arc::new(Some(payload)),
            StatusCode::CREATED,
            Arc::new(HashMap::new()),
        )
        .await
        .into_response();
        let (status, body) = response_status_and_body(response).await;
        assert_eq!(status, StatusCode::CREATED);
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("message"));
        assert!(body_str.contains("test"));
    }

    #[tokio::test]
    async fn handle_custom_route_skips_invalid_headers() {
        let mut headers = HashMap::new();
        headers.insert("x-valid".to_string(), "ok".to_string());
        headers.insert("invalid\nheader".to_string(), "value".to_string());
        headers.insert("key".to_string(), "invalid\x00value".to_string());
        let response = handle_custom_route(Arc::new(None), StatusCode::OK, Arc::new(headers))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("x-valid")
                .and_then(|v| v.to_str().ok()),
            Some("ok")
        );
    }

    #[tokio::test]
    async fn handle_custom_route_status_propagation() {
        let response = handle_custom_route(
            Arc::new(None),
            StatusCode::NOT_FOUND,
            Arc::new(HashMap::new()),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn handle_sequence_route_cycles_correctly() {
        use crate::structs::{Data, Format, JsonOrString};

        let sequence = vec![
            SequenceResponse {
                data: Some(Data {
                    format: Format::Json,
                    payload: Some(JsonOrString::Json(serde_json::json!({"n": 1}))),
                }),
                status: Some(200),
                headers: None,
            },
            SequenceResponse {
                data: Some(Data {
                    format: Format::Json,
                    payload: Some(JsonOrString::Json(serde_json::json!({"n": 2}))),
                }),
                status: Some(200),
                headers: None,
            },
        ];
        let counters = Arc::new(Mutex::new(HashMap::new()));

        let r1 = handle_sequence_route("seq".to_string(), sequence.clone(), counters.clone())
            .await
            .into_response();
        let (_, b1) = response_status_and_body(r1).await;
        assert!(String::from_utf8_lossy(&b1).contains("1"));

        let r2 = handle_sequence_route("seq".to_string(), sequence.clone(), counters.clone())
            .await
            .into_response();
        let (_, b2) = response_status_and_body(r2).await;
        assert!(String::from_utf8_lossy(&b2).contains("2"));

        let r3 = handle_sequence_route("seq".to_string(), sequence.clone(), counters.clone())
            .await
            .into_response();
        let (_, b3) = response_status_and_body(r3).await;
        assert!(
            String::from_utf8_lossy(&b3).contains("2"),
            "after sequence exhausted, repeats last"
        );
    }

    #[tokio::test]
    async fn handle_sequence_route_empty_sequence_returns_default() {
        let sequence = vec![];
        let counters = Arc::new(Mutex::new(HashMap::new()));
        let response = handle_sequence_route("empty".to_string(), sequence, counters)
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
