use axum::http::HeaderValue;
use std::{fmt, str::FromStr};
use uuid::Uuid;

/// HTTP header name for request correlation.
pub const HEADER_NAME: &str = "X-Request-Id";

/// A UUID-based identifier for correlating logs and traces to a single request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestId(String);

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestId {
    /// Creates a new random request ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Returns the request ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns this identifier as an HTTP header value.
    ///
    /// # Panics
    ///
    /// Panics if the identifier cannot be converted to an HTTP header,
    /// which should never happen because it is UUID-based.
    pub fn header_value(&self) -> HeaderValue {
        HeaderValue::from_str(self.as_str()).expect("valid request id")
    }

    /// Parses a request ID from an HTTP header value.
    ///
    /// Returns `None` if the header value is not a valid UUID.
    pub fn from_header_value(value: &HeaderValue) -> Option<Self> {
        value
            .to_str()
            .ok()
            .and_then(|value| value.parse::<RequestId>().ok())
    }

    /// Records this request ID on the current tracing span.
    pub fn record(&self) {
        tracing::Span::current().record("request_id", tracing::field::display(self));
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for RequestId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _ = Uuid::parse_str(s)?;
        Ok(Self(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_header_value() {
        let id = RequestId::new();
        let header = id.header_value();
        let parsed = RequestId::from_header_value(&header).expect("should parse");
        assert_eq!(parsed, id);
    }

    #[test]
    fn rejects_invalid_header() {
        let header = HeaderValue::from_static("not-a-uuid");
        assert!(RequestId::from_header_value(&header).is_none());
    }

    #[test]
    fn default_creates_new_id() {
        let id1 = RequestId::default();
        let id2 = RequestId::default();
        assert_ne!(id1, id2);
    }
}
