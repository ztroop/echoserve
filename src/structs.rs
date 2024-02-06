use clap::Parser;
use serde::{Deserialize, Serialize};

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
    pub data: serde_json::Value,
    pub status: u16,
}

