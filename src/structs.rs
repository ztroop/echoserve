use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short = 'p')]
    pub port: u16,
    #[arg(short = 'a')]
    pub address: Option<String>,
    #[arg(short = 'c')]
    pub config: Option<String>,
    #[arg(short = 'l')]
    pub latency: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub name: String,
    pub endpoint: String,
    #[serde(default)]
    pub method: HttpMethod,
    pub data: Option<serde_json::Value>,
    pub status: u16,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    #[default]
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Debug, Clone)]
pub struct LatencyMiddleware<S> {
    pub inner: S,
    pub delay: Duration,
}
