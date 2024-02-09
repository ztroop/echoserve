use axum::{http::StatusCode, Router};
use clap::Parser;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

mod handlers;
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
        // Load and parse the YAML file
        let configs = load_yaml_config(&config_path);
        for config in configs {
            tracing::info!(
                "Loaded config for ({}) uri={}",
                config.name,
                config.endpoint
            );
            let route_data = Arc::new(config.data.unwrap_or(Value::Null));
            let route_headers =
                Arc::new(config.headers.unwrap_or(HashMap::<String, String>::new()));
            let status_code = StatusCode::from_u16(config.status).unwrap_or(StatusCode::OK);
            app = route_with_method!(app, config.method, &config.endpoint, move || {
                handlers::handle_custom_route(
                    route_data.clone(),
                    status_code,
                    route_headers.clone(),
                )
            });
        }
    } else {
        // Default behaviour
        app = app.route(
            "/*path",
            axum::routing::get(handlers::default_handler)
                .post(handlers::default_handler)
                .put(handlers::default_handler)
                .patch(handlers::default_handler)
                .delete(handlers::default_handler),
        );
    }

    // Add middleware
    app = app.layer(
        TraceLayer::new_for_http()
            .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
            .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
    );

    // Start server
    let addr = format!(
        "{}:{}",
        args.address.unwrap_or("127.0.0.1".to_string()),
        args.port
    );
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("🚀 Listening on {}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

// Load YAML file and return a vector of endpoint configurations
fn load_yaml_config(file_path: &str) -> Vec<structs::EndpointConfig> {
    let file_content = std::fs::read_to_string(file_path).expect("Unable to read file");
    serde_yaml::from_str(&file_content).expect("Unable to parse YAML")
}
