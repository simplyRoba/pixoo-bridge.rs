use serde_json::Value;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PixooError {
    Http(reqwest::Error),
    HttpStatus(u16),
    InvalidBaseUrl(String),
    InvalidResponse(String),
    MissingErrorCode,
    InvalidErrorCode(Value),
    DeviceError { code: i64, payload: Value },
}

impl fmt::Display for PixooError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PixooError::Http(err) => write!(formatter, "http error: {err}"),
            PixooError::HttpStatus(status) => write!(formatter, "unexpected HTTP status: {status}"),
            PixooError::InvalidBaseUrl(message) => {
                write!(formatter, "invalid base url: {message}")
            }
            PixooError::InvalidResponse(message) => {
                write!(formatter, "invalid response: {message}")
            }
            PixooError::MissingErrorCode => formatter.write_str("missing error_code in response"),
            PixooError::InvalidErrorCode(value) => {
                write!(formatter, "invalid error_code value: {value}")
            }
            PixooError::DeviceError { code, .. } => {
                write!(formatter, "device returned error_code {code}")
            }
        }
    }
}

impl Error for PixooError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PixooError::Http(err) => Some(err),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for PixooError {
    fn from(err: reqwest::Error) -> Self {
        PixooError::Http(err)
    }
}
