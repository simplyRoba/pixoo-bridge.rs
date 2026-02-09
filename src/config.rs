use crate::pixoo::PixooClientConfig;
use std::{env, error::Error, fmt, time::Duration};
use tracing::warn;

const DEFAULT_LISTENER_PORT: u16 = 4000;
const MIN_LISTENER_PORT: u16 = 1024;
const MAX_LISTENER_PORT: u16 = 65535;
const DEFAULT_PIXOO_TIMEOUT_MS: u64 = 10_000;
const DEFAULT_ANIMATION_SPEED_FACTOR: f64 = 1.4;
const DEFAULT_MAX_IMAGE_SIZE: usize = 5 * 1024 * 1024; // 5 MB

/// Source for configuration values.
///
/// This trait abstracts environment variable access, allowing configuration
/// functions to be tested with mock values instead of manipulating global
/// process state. In production, use [`EnvConfigSource`] which reads from
/// `std::env`. In tests, implement this trait with a simple `HashMap`.
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
    pub animation_speed_factor: f64,
    pub max_image_size: usize,
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
        let animation_speed_factor = resolve_animation_speed_factor(source);
        let max_image_size = resolve_max_image_size(source);

        Ok(Self {
            pixoo_base_url,
            pixoo_client,
            health_forward,
            listener_port,
            animation_speed_factor,
            max_image_size,
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

fn resolve_animation_speed_factor(source: &impl ConfigSource) -> f64 {
    match source.get("PIXOO_ANIMATION_SPEED_FACTOR") {
        Some(raw) => {
            let value = raw.trim();
            match value.parse::<f64>() {
                Ok(factor) if factor > 0.0 && factor.is_finite() => factor,
                _ => {
                    warn!(
                        provided = %value,
                        default = DEFAULT_ANIMATION_SPEED_FACTOR,
                        "Invalid PIXOO_ANIMATION_SPEED_FACTOR; falling back to default"
                    );
                    DEFAULT_ANIMATION_SPEED_FACTOR
                }
            }
        }
        None => DEFAULT_ANIMATION_SPEED_FACTOR,
    }
}

fn resolve_max_image_size(source: &impl ConfigSource) -> usize {
    match source.get("PIXOO_BRIDGE_MAX_IMAGE_SIZE") {
        Some(raw) => match parse_byte_size(raw.trim()) {
            Some(size) if size > 0 => size,
            _ => {
                warn!(
                    provided = %raw.trim(),
                    default = DEFAULT_MAX_IMAGE_SIZE,
                    "Invalid PIXOO_BRIDGE_MAX_IMAGE_SIZE; falling back to default"
                );
                DEFAULT_MAX_IMAGE_SIZE
            }
        },
        None => DEFAULT_MAX_IMAGE_SIZE,
    }
}

/// Parses a human-readable byte size string into bytes.
///
/// Accepts formats like `5MB`, `128KB`, `1024B`, `5M`, `128K` (case-insensitive).
/// Uses binary units: 1 KB = 1024 bytes, 1 MB = 1024 * 1024 bytes.
fn parse_byte_size(input: &str) -> Option<usize> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    let upper = input.to_ascii_uppercase();

    // Find where the numeric part ends and the suffix begins
    let num_end = upper
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(upper.len());

    let (num_str, suffix) = upper.split_at(num_end);
    let number: usize = num_str.parse().ok()?;

    let multiplier: usize = match suffix {
        "" | "B" => 1,
        "K" | "KB" => 1024,
        "M" | "MB" => 1024 * 1024,
        "G" | "GB" => 1024 * 1024 * 1024,
        _ => return None,
    };

    number.checked_mul(multiplier)
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

    // --- parse_byte_size ---

    #[test]
    fn parse_byte_size_megabytes() {
        assert_eq!(parse_byte_size("5MB"), Some(5 * 1024 * 1024));
    }

    #[test]
    fn parse_byte_size_megabytes_without_b() {
        assert_eq!(parse_byte_size("5M"), Some(5 * 1024 * 1024));
    }

    #[test]
    fn parse_byte_size_kilobytes() {
        assert_eq!(parse_byte_size("128KB"), Some(128 * 1024));
    }

    #[test]
    fn parse_byte_size_kilobytes_without_b() {
        assert_eq!(parse_byte_size("128K"), Some(128 * 1024));
    }

    #[test]
    fn parse_byte_size_bytes() {
        assert_eq!(parse_byte_size("1024B"), Some(1024));
    }

    #[test]
    fn parse_byte_size_plain_number() {
        assert_eq!(parse_byte_size("2048"), Some(2048));
    }

    #[test]
    fn parse_byte_size_case_insensitive() {
        assert_eq!(parse_byte_size("10mb"), Some(10 * 1024 * 1024));
        assert_eq!(parse_byte_size("10Mb"), Some(10 * 1024 * 1024));
    }

    #[test]
    fn parse_byte_size_gigabytes() {
        assert_eq!(parse_byte_size("1GB"), Some(1024 * 1024 * 1024));
    }

    #[test]
    fn parse_byte_size_invalid_returns_none() {
        assert_eq!(parse_byte_size("lots"), None);
        assert_eq!(parse_byte_size(""), None);
        assert_eq!(parse_byte_size("MB"), None);
        assert_eq!(parse_byte_size("5TB"), None);
    }

    // --- animation speed factor ---

    #[test]
    fn animation_speed_factor_defaults_when_missing() {
        let config = MockConfig::new();
        let factor = resolve_animation_speed_factor(&config);
        assert!((factor - DEFAULT_ANIMATION_SPEED_FACTOR).abs() < f64::EPSILON);
    }

    #[test]
    fn animation_speed_factor_uses_valid_override() {
        let config = MockConfig::new().with("PIXOO_ANIMATION_SPEED_FACTOR", "2.0");
        let factor = resolve_animation_speed_factor(&config);
        assert!((factor - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn animation_speed_factor_falls_back_on_invalid() {
        let config = MockConfig::new().with("PIXOO_ANIMATION_SPEED_FACTOR", "abc");
        let factor = resolve_animation_speed_factor(&config);
        assert!((factor - DEFAULT_ANIMATION_SPEED_FACTOR).abs() < f64::EPSILON);
    }

    #[test]
    fn animation_speed_factor_falls_back_on_zero() {
        let config = MockConfig::new().with("PIXOO_ANIMATION_SPEED_FACTOR", "0");
        let factor = resolve_animation_speed_factor(&config);
        assert!((factor - DEFAULT_ANIMATION_SPEED_FACTOR).abs() < f64::EPSILON);
    }

    #[test]
    fn animation_speed_factor_falls_back_on_negative() {
        let config = MockConfig::new().with("PIXOO_ANIMATION_SPEED_FACTOR", "-1.0");
        let factor = resolve_animation_speed_factor(&config);
        assert!((factor - DEFAULT_ANIMATION_SPEED_FACTOR).abs() < f64::EPSILON);
    }

    // --- max image size ---

    #[test]
    fn max_image_size_defaults_when_missing() {
        let config = MockConfig::new();
        let size = resolve_max_image_size(&config);
        assert_eq!(size, DEFAULT_MAX_IMAGE_SIZE);
    }

    #[test]
    fn max_image_size_uses_valid_megabyte_override() {
        let config = MockConfig::new().with("PIXOO_BRIDGE_MAX_IMAGE_SIZE", "10MB");
        let size = resolve_max_image_size(&config);
        assert_eq!(size, 10 * 1024 * 1024);
    }

    #[test]
    fn max_image_size_uses_valid_kilobyte_override() {
        let config = MockConfig::new().with("PIXOO_BRIDGE_MAX_IMAGE_SIZE", "128K");
        let size = resolve_max_image_size(&config);
        assert_eq!(size, 128 * 1024);
    }

    #[test]
    fn max_image_size_falls_back_on_invalid() {
        let config = MockConfig::new().with("PIXOO_BRIDGE_MAX_IMAGE_SIZE", "lots");
        let size = resolve_max_image_size(&config);
        assert_eq!(size, DEFAULT_MAX_IMAGE_SIZE);
    }
}
