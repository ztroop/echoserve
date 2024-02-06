use axum::response::IntoResponse;
use axum::{http::StatusCode, response::Json, routing::get, Router};
use clap::Parser;
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

mod structs;

#[tokio::main]
async fn main() {
    let args = structs::Args::parse();

    let mut app = Router::new();

    if let Some(config_path) = args.config {
        // Load and parse the YAML file
        let configs = load_yaml_config(&config_path);
        for config in configs {
            let route_data = Arc::new(config.data.unwrap_or(Value::Null));
            let route_headers =
                Arc::new(config.headers.unwrap_or(HashMap::<String, String>::new()));
            let status_code = StatusCode::from_u16(config.status).unwrap_or(StatusCode::OK);
            app = app.route(
                &config.endpoint,
                get(move || {
                    handle_custom_route(route_data.clone(), status_code, route_headers.clone())
                }),
            );
        }
    } else {
        // Default behavior
        app = app.route("/", get(default_handler));
    }

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Load YAML file and return a vector of endpoint configurations
fn load_yaml_config(file_path: &str) -> Vec<structs::EndpointConfig> {
    let file_content = std::fs::read_to_string(file_path).expect("Unable to read file");
    serde_yaml::from_str(&file_content).expect("Unable to parse YAML")
}

// Default handler for no YAML configuration
async fn default_handler() -> &'static str {
    "OK"
}

// Handler for custom route
async fn handle_custom_route(
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
