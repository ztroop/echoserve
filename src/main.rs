use axum::{http::StatusCode, Router};
use clap::Parser;
use std::sync::Arc;
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};
use tokio::sync::Mutex;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

use crate::handlers::default_handler;
use crate::structs::Data;

mod handlers;
mod latency;
mod macros;
mod structs;

#[derive(Debug)]
enum ConfigError {
    Io(String, std::io::Error),
    Yaml(String, serde_yaml_ng::Error),
    Validation(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(path, e) => {
                write!(f, "Unable to read config file '{}': {}", path, e)
            }
            ConfigError::Yaml(path, e) => {
                write!(f, "Unable to parse YAML in '{}': {}", path, e)
            }
            ConfigError::Validation(msg) => write!(f, "Invalid config: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(_, e) => Some(e),
            ConfigError::Yaml(_, e) => Some(e),
            ConfigError::Validation(_) => None,
        }
    }
}

fn method_as_str(m: &structs::HttpMethod) -> &'static str {
    match m {
        structs::HttpMethod::Get => "GET",
        structs::HttpMethod::Post => "POST",
        structs::HttpMethod::Put => "PUT",
        structs::HttpMethod::Delete => "DELETE",
        structs::HttpMethod::Patch => "PATCH",
    }
}

fn validate_config(configs: &[structs::EndpointConfig]) -> Result<(), ConfigError> {
    if configs.is_empty() {
        tracing::warn!("Config file contains no endpoints");
    }
    let mut seen = HashSet::new();
    for config in configs {
        if !config.endpoint.starts_with('/') {
            return Err(ConfigError::Validation(format!(
                "Endpoint '{}' ({}) must start with '/'",
                config.name, config.endpoint
            )));
        }
        if let Some(sequence) = &config.sequence {
            if sequence.is_empty() {
                return Err(ConfigError::Validation(format!(
                    "Endpoint '{}' ({}) has an empty sequence",
                    config.name, config.endpoint
                )));
            }
        }
        let key = format!("{}:{}", method_as_str(&config.method), config.endpoint);
        if !seen.insert(key) {
            tracing::warn!(
                "Duplicate endpoint {} {} - later definition will override",
                method_as_str(&config.method),
                config.endpoint
            );
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .compact()
        .init();

    let args = structs::Args::parse();
    let mut app = Router::new();

    if let Some(config_path) = args.config {
        let configs = match load_yaml_config(&config_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Config error: {}", e);
                std::process::exit(1);
            }
        };
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
        args.address.as_deref().unwrap_or("127.0.0.1"),
        args.port.unwrap_or(8080)
    );
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!(
                "Failed to bind to {}. The address is already in use or unavailable. Please check if another instance is running or use a different port.\nError: {}",
                &addr, e
            );
            std::process::exit(1);
        }
    };
    tracing::info!("🚀 Listening on {}", &addr);
    if let Err(e) = axum::serve(listener, app.into_make_service()).await {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }
}

fn load_yaml_config(file_path: &str) -> Result<Vec<structs::EndpointConfig>, ConfigError> {
    let file_content = std::fs::read_to_string(file_path)
        .map_err(|e| ConfigError::Io(file_path.to_string(), e))?;
    let configs: Vec<structs::EndpointConfig> = serde_yaml_ng::from_str(&file_content)
        .map_err(|e| ConfigError::Yaml(file_path.to_string(), e))?;
    validate_config(&configs)?;
    Ok(configs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn load_yaml_config_valid_file() {
        let configs = load_yaml_config("examples/config.yml").unwrap();
        assert!(!configs.is_empty());
        assert_eq!(configs[0].name, "Example Endpoint 1");
        assert_eq!(configs[0].endpoint, "/example1");
    }

    #[test]
    fn load_yaml_config_missing_file() {
        let result = load_yaml_config("nonexistent/config.yml");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Io(_, _)));
    }

    #[test]
    fn load_yaml_config_invalid_yaml() {
        let tmp = NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), "invalid: yaml: [[[").unwrap();
        let result = load_yaml_config(&tmp.path().to_string_lossy());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Yaml(_, _)));
    }

    #[test]
    fn load_yaml_config_rejects_empty_sequence() {
        let tmp = NamedTempFile::new().unwrap();
        let yaml = r#"
- name: "EmptySeq"
  endpoint: "/empty"
  method: "GET"
  sequence: []
"#;
        std::fs::write(tmp.path(), yaml).unwrap();
        let result = load_yaml_config(&tmp.path().to_string_lossy());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Validation(_)));
    }

    #[test]
    fn load_yaml_config_rejects_endpoint_without_leading_slash() {
        let tmp = NamedTempFile::new().unwrap();
        let yaml = r#"
- name: "BadPath"
  endpoint: "noleadingslash"
  method: "GET"
"#;
        std::fs::write(tmp.path(), yaml).unwrap();
        let result = load_yaml_config(&tmp.path().to_string_lossy());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Validation(_)));
    }

    #[test]
    fn parse_minimal_endpoint() {
        let yaml = r#"
- name: "Minimal"
  endpoint: "/minimal"
"#;
        let configs: Vec<structs::EndpointConfig> = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].name, "Minimal");
        assert_eq!(configs[0].endpoint, "/minimal");
        assert!(matches!(configs[0].method, structs::HttpMethod::Get));
    }
}
