use crate::pixoo::PixooClientConfig;
use std::{env, error::Error, fmt, time::Duration};
use tracing::warn;

const DEFAULT_LISTENER_PORT: u16 = 4000;
const MIN_LISTENER_PORT: u16 = 1024;
const MAX_LISTENER_PORT: u16 = 65535;
const DEFAULT_PIXOO_TIMEOUT_MS: u64 = 10_000;

/// Source for configuration values.
///
/// This trait abstracts environment variable access, allowing configuration
/// functions to be tested with mock values instead of manipulating global
/// process state. In production, use [`EnvConfigSource`] which reads from
/// `std::env`. In tests, implement this trait with a simple HashMap.
pub trait ConfigSource {
    fn get(&self, key: &str) -> Option<String>;
}

/// Configuration source that reads from environment variables.
pub struct EnvConfigSource;

impl ConfigSource for EnvConfigSource {
    fn get(&self, key: &str) -> Option<String> {
        env::var(key).ok()
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub pixoo_base_url: String,
    pub pixoo_client: PixooClientConfig,
    pub health_forward: bool,
    pub listener_port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    MissingPixooBaseUrl,
    InvalidPixooBaseUrl(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingPixooBaseUrl => {
                write!(f, "PIXOO_BASE_URL is required but was not set")
            }
            ConfigError::InvalidPixooBaseUrl(err) => {
                write!(f, "PIXOO_BASE_URL is invalid: {err}")
            }
        }
    }
}

impl Error for ConfigError {}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from(&EnvConfigSource)
    }

    pub fn load_from(source: &impl ConfigSource) -> Result<Self, ConfigError> {
        let health_forward = read_bool(source, "PIXOO_BRIDGE_HEALTH_FORWARD", true);
        let pixoo_base_url = resolve_pixoo_base_url(source)?;
        let pixoo_client = resolve_pixoo_client_config(source);
        let listener_port = resolve_listener_port(source);

        Ok(Self {
            pixoo_base_url,
            pixoo_client,
            health_forward,
            listener_port,
        })
    }
}

/// `PIXOO_BASE_URL` used to be optional and led to runtime 503s; now it is required at startup.
fn resolve_pixoo_base_url(source: &impl ConfigSource) -> Result<String, ConfigError> {
    let raw = source
        .get("PIXOO_BASE_URL")
        .ok_or(ConfigError::MissingPixooBaseUrl)?;
    let value = raw.trim();
    if value.is_empty() {
        return Err(ConfigError::MissingPixooBaseUrl);
    }
    reqwest::Url::parse(value).map_err(|err| ConfigError::InvalidPixooBaseUrl(err.to_string()))?;
    Ok(value.to_string())
}

fn resolve_pixoo_client_config(source: &impl ConfigSource) -> PixooClientConfig {
    let timeout = source
        .get("PIXOO_TIMEOUT_MS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or_else(
            || Duration::from_millis(DEFAULT_PIXOO_TIMEOUT_MS),
            Duration::from_millis,
        );
    let defaults = PixooClientConfig::default();
    PixooClientConfig::new(timeout, defaults.retries, defaults.backoff)
}

fn read_bool(source: &impl ConfigSource, key: &str, default: bool) -> bool {
    match source.get(key) {
        Some(value) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => default,
        },
        None => default,
    }
}

fn resolve_listener_port(source: &impl ConfigSource) -> u16 {
    match source.get("PIXOO_BRIDGE_PORT") {
        Some(raw) => {
            let value = raw.trim();
            match value.parse::<u16>() {
                Ok(port) if (MIN_LISTENER_PORT..=MAX_LISTENER_PORT).contains(&port) => port,
                _ => {
                    warn!(
                        provided = %value,
                        min = MIN_LISTENER_PORT,
                        max = MAX_LISTENER_PORT,
                        default_port = DEFAULT_LISTENER_PORT,
                        "Invalid PIXOO_BRIDGE_PORT; falling back to default port {}",
                        DEFAULT_LISTENER_PORT
                    );
                    DEFAULT_LISTENER_PORT
                }
            }
        }
        None => DEFAULT_LISTENER_PORT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockConfig(HashMap<&'static str, &'static str>);

    impl MockConfig {
        fn new() -> Self {
            Self(HashMap::new())
        }

        fn with(mut self, key: &'static str, value: &'static str) -> Self {
            self.0.insert(key, value);
            self
        }
    }

    impl ConfigSource for MockConfig {
        fn get(&self, key: &str) -> Option<String> {
            self.0.get(key).map(|s| (*s).to_string())
        }
    }

    #[test]
    fn pixoo_base_url_missing_is_error() {
        let config = MockConfig::new();
        let err = resolve_pixoo_base_url(&config).expect_err("expected missing base url error");
        assert_eq!(err, ConfigError::MissingPixooBaseUrl);
    }

    #[test]
    fn pixoo_base_url_invalid_is_error() {
        let config = MockConfig::new().with("PIXOO_BASE_URL", "not a url");
        let err = resolve_pixoo_base_url(&config).expect_err("expected invalid base url error");
        assert!(matches!(err, ConfigError::InvalidPixooBaseUrl(_)));
    }

    #[test]
    fn pixoo_base_url_valid_is_loaded() {
        let config = MockConfig::new().with("PIXOO_BASE_URL", "http://127.0.0.1");
        let value = resolve_pixoo_base_url(&config).expect("expected base url");
        assert_eq!(value, "http://127.0.0.1");
    }

    #[test]
    fn pixoo_timeout_uses_env_override() {
        let config = MockConfig::new().with("PIXOO_TIMEOUT_MS", "250");
        let client_config = resolve_pixoo_client_config(&config);
        assert_eq!(client_config.timeout, Duration::from_millis(250));
    }

    #[test]
    fn listener_port_defaults_to_4000_when_env_missing() {
        let config = MockConfig::new();
        let port = resolve_listener_port(&config);
        assert_eq!(port, DEFAULT_LISTENER_PORT);
    }

    #[test]
    fn listener_port_uses_custom_override_when_valid() {
        let config = MockConfig::new().with("PIXOO_BRIDGE_PORT", "5050");
        let port = resolve_listener_port(&config);
        assert_eq!(port, 5050);
    }

    #[test]
    fn listener_port_falls_back_on_invalid_values() {
        let config = MockConfig::new().with("PIXOO_BRIDGE_PORT", "not-a-port");
        let port = resolve_listener_port(&config);
        assert_eq!(port, DEFAULT_LISTENER_PORT);
    }
}
