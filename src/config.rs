use pixoo_bridge::pixoo::PixooClientConfig;
use std::{env, error::Error, fmt, time::Duration};
use tracing::warn;

const DEFAULT_LISTENER_PORT: u16 = 4000;
const MIN_LISTENER_PORT: u16 = 1024;
const MAX_LISTENER_PORT: u16 = 65535;
const DEFAULT_PIXOO_TIMEOUT_MS: u64 = 10_000;

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
        let health_forward = read_bool_env("PIXOO_BRIDGE_HEALTH_FORWARD", true);
        let pixoo_base_url = resolve_pixoo_base_url()?;
        let pixoo_client = resolve_pixoo_client_config();
        let listener_port = resolve_listener_port();

        Ok(Self {
            pixoo_base_url,
            pixoo_client,
            health_forward,
            listener_port,
        })
    }
}

/// `PIXOO_BASE_URL` used to be optional and led to runtime 503s; now it is required at startup.
fn resolve_pixoo_base_url() -> Result<String, ConfigError> {
    let raw = env::var("PIXOO_BASE_URL").map_err(|_| ConfigError::MissingPixooBaseUrl)?;
    let value = raw.trim();
    if value.is_empty() {
        return Err(ConfigError::MissingPixooBaseUrl);
    }
    reqwest::Url::parse(value).map_err(|err| ConfigError::InvalidPixooBaseUrl(err.to_string()))?;
    Ok(value.to_string())
}

fn resolve_pixoo_client_config() -> PixooClientConfig {
    let timeout = env::var("PIXOO_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map_or_else(
            || Duration::from_millis(DEFAULT_PIXOO_TIMEOUT_MS),
            Duration::from_millis,
        );
    let defaults = PixooClientConfig::default();
    PixooClientConfig::new(timeout, defaults.retries, defaults.backoff)
}

fn read_bool_env(key: &str, default: bool) -> bool {
    match env::var(key) {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => default,
        },
        Err(_) => default,
    }
}

fn resolve_listener_port() -> u16 {
    match env::var("PIXOO_BRIDGE_PORT") {
        Ok(raw) => {
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
        Err(_) => DEFAULT_LISTENER_PORT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("lock")
    }

    fn with_env_var<T>(key: &str, value: Option<&str>, f: impl FnOnce() -> T) -> T {
        let _guard = env_lock();
        let original = env::var(key).ok();
        match value {
            Some(v) => unsafe { env::set_var(key, v) },
            None => unsafe { env::remove_var(key) },
        }
        let result = f();
        match original {
            Some(v) => unsafe { env::set_var(key, v) },
            None => unsafe { env::remove_var(key) },
        }
        result
    }

    #[test]
    fn pixoo_base_url_missing_is_error() {
        let err = with_env_var("PIXOO_BASE_URL", None, resolve_pixoo_base_url)
            .expect_err("expected missing base url error");
        assert_eq!(err, ConfigError::MissingPixooBaseUrl);
    }

    #[test]
    fn pixoo_base_url_invalid_is_error() {
        let err = with_env_var("PIXOO_BASE_URL", Some("not a url"), resolve_pixoo_base_url)
            .expect_err("expected invalid base url error");
        assert!(matches!(err, ConfigError::InvalidPixooBaseUrl(_)));
    }

    #[test]
    fn pixoo_base_url_valid_is_loaded() {
        let value = with_env_var(
            "PIXOO_BASE_URL",
            Some("http://127.0.0.1"),
            resolve_pixoo_base_url,
        )
        .expect("expected base url");
        assert_eq!(value, "http://127.0.0.1");
    }

    #[test]
    fn pixoo_timeout_uses_env_override() {
        let config = with_env_var("PIXOO_TIMEOUT_MS", Some("250"), resolve_pixoo_client_config);
        assert_eq!(config.timeout, Duration::from_millis(250));
    }

    #[test]
    fn listener_port_defaults_to_4000_when_env_missing() {
        let port = with_env_var("PIXOO_BRIDGE_PORT", None, resolve_listener_port);
        assert_eq!(port, DEFAULT_LISTENER_PORT);
    }

    #[test]
    fn listener_port_uses_custom_override_when_valid() {
        let port = with_env_var("PIXOO_BRIDGE_PORT", Some("5050"), resolve_listener_port);
        assert_eq!(port, 5050);
    }

    #[test]
    fn listener_port_falls_back_on_invalid_values() {
        let port = with_env_var(
            "PIXOO_BRIDGE_PORT",
            Some("not-a-port"),
            resolve_listener_port,
        );
        assert_eq!(port, DEFAULT_LISTENER_PORT);
    }
}
