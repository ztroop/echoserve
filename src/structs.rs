use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short = 'p')]
    pub port: u16,
    #[arg(short = 'c')]
    pub config: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub name: String,
    pub endpoint: String,
    pub data: Option<serde_json::Value>,
    pub status: u16,
    pub headers: Option<HashMap<String, String>>,
}
