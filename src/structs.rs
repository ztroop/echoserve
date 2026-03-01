use clap::Parser;
use serde::{de::Visitor, Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fmt, time::Duration};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(help = "Port number to listen on. (Default: 8080)")]
    #[arg(short = 'p')]
    pub port: Option<u16>,
    #[clap(help = "Address to listen on. (Default: 127.0.0.1)")]
    #[arg(short = 'a')]
    pub address: Option<String>,
    #[clap(help = "Optional path to a YAML configuration file.")]
    #[arg(short = 'c')]
    pub config: Option<String>,
    #[clap(help = "Simulated latency in milliseconds. (Default: 0)")]
    #[arg(short = 'l')]
    pub latency: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SequenceResponse {
    pub data: Option<Data>,
    pub status: Option<u16>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EndpointConfig {
    pub name: String,
    pub endpoint: String,
    #[serde(default)]
    pub method: HttpMethod,
    pub data: Option<Data>,
    pub status: Option<u16>,
    pub headers: Option<HashMap<String, String>>,
    pub sequence: Option<Vec<SequenceResponse>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Data {
    #[serde(default)]
    pub format: Format,
    pub payload: Option<JsonOrString>,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            format: Format::Json,
            payload: Some(JsonOrString::Json(serde_json::Value::Null)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    #[default]
    Json,
    Xml,
    Text,
    Html,
}

impl Format {
    pub fn as_content_type(&self) -> &str {
        match self {
            Format::Json => "application/json",
            Format::Xml => "application/xml",
            Format::Text => "text/plain",
            Format::Html => "text/html",
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum JsonOrString {
    Json(serde_json::Value),
    Str(String),
}

impl<'de> Deserialize<'de> for JsonOrString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StringOrJsonValue;

        impl<'de> Visitor<'de> for StringOrJsonValue {
            type Value = JsonOrString;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or a JSON value")
            }

            fn visit_str<E>(self, value: &str) -> Result<JsonOrString, E>
            where
                E: serde::de::Error,
            {
                Ok(JsonOrString::Str(value.to_owned()))
            }

            fn visit_map<M>(self, map: M) -> Result<JsonOrString, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let json = Value::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                Ok(JsonOrString::Json(json))
            }
        }

        deserializer.deserialize_any(StringOrJsonValue)
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_as_content_type_json() {
        assert_eq!(Format::Json.as_content_type(), "application/json");
    }

    #[test]
    fn format_as_content_type_xml() {
        assert_eq!(Format::Xml.as_content_type(), "application/xml");
    }

    #[test]
    fn format_as_content_type_text() {
        assert_eq!(Format::Text.as_content_type(), "text/plain");
    }

    #[test]
    fn format_as_content_type_html() {
        assert_eq!(Format::Html.as_content_type(), "text/html");
    }

    #[test]
    fn json_or_string_deserialize_string() {
        let yaml = r#""hello world""#;
        let result: JsonOrString = serde_yaml_ng::from_str(yaml).unwrap();
        match result {
            JsonOrString::Str(s) => assert_eq!(s, "hello world"),
            _ => panic!("Expected Str variant"),
        }
    }

    #[test]
    fn json_or_string_deserialize_json() {
        let yaml = r#"
foo: bar
nested:
  a: 1
"#;
        let result: JsonOrString = serde_yaml_ng::from_str(yaml).unwrap();
        match result {
            JsonOrString::Json(v) => {
                assert_eq!(v.get("foo").and_then(|v| v.as_str()), Some("bar"));
            }
            _ => panic!("Expected Json variant"),
        }
    }

    #[test]
    fn http_method_default() {
        let method: HttpMethod = Default::default();
        assert!(matches!(method, HttpMethod::Get));
    }

    #[test]
    fn http_method_deserialize() {
        let method: HttpMethod = serde_yaml_ng::from_str(r#""POST""#).unwrap();
        assert!(matches!(method, HttpMethod::Post));
    }

    #[test]
    fn data_default() {
        let data = Data::default();
        assert!(matches!(data.format, Format::Json));
        assert!(data.payload.is_some());
        match data.payload.as_ref().unwrap() {
            JsonOrString::Json(v) => assert!(v.is_null()),
            _ => panic!("Expected Json(Null)"),
        }
    }
}
