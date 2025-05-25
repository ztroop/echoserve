use axum::{http::StatusCode, Router};
use clap::Parser;
use std::sync::Arc;
use std::{collections::HashMap, time::Duration};
use tokio::sync::Mutex;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

use crate::handlers::default_handler;
use crate::structs::Data;

mod handlers;
mod latency;
mod macros;
mod structs;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .compact()
        .init();

    let args = structs::Args::parse();
    let mut app = Router::new();

    if let Some(config_path) = args.config {
        let configs = load_yaml_config(&config_path);
        let counters = Arc::new(Mutex::new(HashMap::<String, usize>::new()));
        for config in configs {
            tracing::info!("Loaded endpoint: ({}) uri={}", config.name, config.endpoint);
            if let Some(sequence) = &config.sequence {
                let endpoint = config.endpoint.clone();
                let method = config.method.clone();
                let sequence = sequence.clone();
                let counters = counters.clone();
                app = route_with_method!(app, method, &endpoint, {
                    let endpoint = endpoint.clone();
                    let sequence = sequence.clone();
                    let counters = counters.clone();
                    async move || {
                        handlers::handle_sequence_route(endpoint, sequence, counters).await
                    }
                });
                continue;
            }
            let route_data = config.data.unwrap_or(Data::default());
            let mut route_headers = config.headers.unwrap_or(HashMap::<String, String>::new());
            route_headers.insert(
                "content-type".to_string(),
                route_data.format.as_content_type().to_string(),
            );
            let route_payload = match route_data.payload {
                Some(payload) => match payload {
                    structs::JsonOrString::Json(value) => Some(value),
                    structs::JsonOrString::Str(value) => Some(serde_json::json!(value)),
                },
                None => None,
            };
            let status_code =
                StatusCode::from_u16(config.status.unwrap_or(200)).unwrap_or(StatusCode::OK);
            app = route_with_method!(app, config.method, &config.endpoint, move || {
                handlers::handle_custom_route(
                    Arc::new(route_payload),
                    status_code,
                    Arc::new(route_headers),
                )
            });
        }
    } else {
        app = app.route(
            "/*path",
            axum::routing::get(default_handler)
                .post(default_handler)
                .put(default_handler)
                .patch(default_handler)
                .delete(default_handler),
        );
    }

    app = app
        .layer(latency::with_latency(Duration::from_millis(
            args.latency.unwrap_or(0),
        )))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let addr = format!(
        "{}:{}",
        args.address.unwrap_or("127.0.0.1".to_string()),
        args.port.unwrap_or(8080)
    );
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("🚀 Listening on {}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

fn load_yaml_config(file_path: &str) -> Vec<structs::EndpointConfig> {
    let file_content = std::fs::read_to_string(file_path).expect("Unable to read file");
    serde_yaml::from_str(&file_content).expect("Unable to parse YAML")
}
