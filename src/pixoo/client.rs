use crate::pixoo::command::PixooCommand;
use crate::pixoo::error::PixooError;
use reqwest::header::CONTENT_TYPE;
use serde_json::{Map, Value};
use std::{env, time::Duration};
use tokio::time::sleep;
use tracing::{debug, error};

pub type PixooResponse = Map<String, Value>;

#[derive(Debug, Clone)]
pub struct PixooClient {
    post_url: String,
    get_url: String,
    http: reqwest::Client,
    retries: usize,
    backoff: Duration,
}

impl PixooClient {
    /// Creates a new Pixoo client for the given base URL.
    ///
    /// # Errors
    ///
    /// Returns [`PixooError::InvalidBaseUrl`] if the URL cannot be parsed.
    /// Returns [`PixooError::Http`] if the HTTP client fails to initialize.
    pub fn new(base_url: impl Into<String>) -> Result<Self, PixooError> {
        let base_url = base_url.into();
        let post_url = reqwest::Url::parse(&base_url)
            .map_err(|err| PixooError::InvalidBaseUrl(err.to_string()))
            .map(|mut url| {
                url.set_path("/post");
                url.to_string()
            })?;
        let get_url = reqwest::Url::parse(&base_url)
            .map_err(|err| PixooError::InvalidBaseUrl(err.to_string()))
            .map(|mut url| {
                url.set_path("/get");
                url.to_string()
            })?;
        let http = reqwest::Client::builder()
            .timeout(client_timeout())
            .build()?;

        Ok(Self {
            post_url,
            get_url,
            http,
            retries: 2,
            backoff: Duration::from_millis(200),
        })
    }

    fn build_payload(command: &PixooCommand, mut args: Map<String, Value>) -> Map<String, Value> {
        args.insert(
            "Command".to_string(),
            Value::String(command.as_str().to_string()),
        );
        args
    }

    /// Sends a command to the Pixoo device.
    ///
    /// # Errors
    ///
    /// Returns [`PixooError::Http`] if the request fails due to network issues.
    /// Returns [`PixooError::HttpStatus`] if the device returns a non-2xx status.
    /// Returns [`PixooError::DeviceError`] if the device returns a non-zero error code.
    /// Returns [`PixooError::InvalidResponse`] if the response cannot be parsed.
    /// Returns [`PixooError::MissingErrorCode`] if the response lacks an `error_code` field.
    pub async fn send_command(
        &self,
        command: PixooCommand,
        args: Map<String, Value>,
    ) -> Result<PixooResponse, PixooError> {
        let payload = Self::build_payload(&command, args);
        debug!(command = ?command, payload = ?payload, "sending Pixoo command");

        let response = self.execute_with_retry(&payload).await;
        if let Ok(ref body) = response {
            debug!(command = ?command, response = ?body, "Pixoo command response");
        }
        response
    }

    /// Checks if the Pixoo device is reachable.
    ///
    /// # Errors
    ///
    /// Returns [`PixooError::Http`] if the request fails due to network issues.
    /// Returns [`PixooError::HttpStatus`] if the device returns a non-2xx status.
    pub async fn health_check(&self) -> Result<(), PixooError> {
        self.execute_health_with_retry().await
    }

    async fn execute_with_retry(
        &self,
        payload: &Map<String, Value>,
    ) -> Result<PixooResponse, PixooError> {
        let mut attempt = 0;

        loop {
            match self.execute_once(payload).await {
                Ok(response) => {
                    if attempt > 0 {
                        debug!(
                            attempts = attempt + 1,
                            "Pixoo command succeeded after retries"
                        );
                    }
                    return Ok(response);
                }
                Err(err) => {
                    let retriable = is_retriable(&err);
                    if attempt >= self.retries || !retriable {
                        log_pixoo_error("sending Pixoo command", &err, retriable);
                        return Err(err);
                    }

                    attempt += 1;
                    let delay = self.backoff * u32::try_from(attempt).unwrap_or(u32::MAX);
                    sleep(delay).await;
                }
            }
        }
    }

    async fn execute_health_with_retry(&self) -> Result<(), PixooError> {
        let mut attempt = 0;

        loop {
            match self.execute_health_once().await {
                Ok(()) => {
                    if attempt > 0 {
                        debug!(
                            attempts = attempt + 1,
                            "Pixoo health check succeeded after retries"
                        );
                    }
                    return Ok(());
                }
                Err(err) => {
                    let retriable = is_retriable(&err);
                    if attempt >= self.retries || !retriable {
                        log_pixoo_error("Pixoo health check", &err, retriable);
                        return Err(err);
                    }

                    attempt += 1;
                    let delay = self.backoff * u32::try_from(attempt).unwrap_or(u32::MAX);
                    sleep(delay).await;
                }
            }
        }
    }

    async fn execute_once(
        &self,
        payload: &Map<String, Value>,
    ) -> Result<PixooResponse, PixooError> {
        let response = self
            .http
            .post(&self.post_url)
            .header(CONTENT_TYPE, "application/json")
            .json(payload)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(PixooError::HttpStatus(status.as_u16()));
        }

        parse_response(&body)
    }

    async fn execute_health_once(&self) -> Result<(), PixooError> {
        let response = self.http.get(&self.get_url).send().await?;

        let status = response.status();

        if !status.is_success() {
            return Err(PixooError::HttpStatus(status.as_u16()));
        }

        Ok(())
    }
}

fn is_retriable(err: &PixooError) -> bool {
    match err {
        PixooError::Http(_) => true,
        PixooError::HttpStatus(status) => *status >= 500,
        _ => false,
    }
}

fn parse_response(body: &str) -> Result<PixooResponse, PixooError> {
    let value: Value =
        serde_json::from_str(body).map_err(|err| PixooError::InvalidResponse(err.to_string()))?;
    let mut object = value
        .as_object()
        .cloned()
        .ok_or_else(|| PixooError::InvalidResponse("expected JSON object".to_string()))?;

    let error_code_value = object
        .remove("error_code")
        .ok_or(PixooError::MissingErrorCode)?;
    let error_code = parse_error_code(&error_code_value)?;

    if error_code != 0 {
        return Err(PixooError::DeviceError {
            code: error_code,
            payload: Value::Object(object),
        });
    }

    Ok(object)
}

fn parse_error_code(value: &Value) -> Result<i64, PixooError> {
    match value {
        Value::Number(number) => number
            .as_i64()
            .or_else(|| number.as_u64().and_then(|v| i64::try_from(v).ok()))
            .ok_or_else(|| PixooError::InvalidErrorCode(value.clone())),
        Value::String(text) => text
            .parse::<i64>()
            .map_err(|_| PixooError::InvalidErrorCode(value.clone())),
        _ => Err(PixooError::InvalidErrorCode(value.clone())),
    }
}

fn log_pixoo_error(context: &str, err: &PixooError, retriable: bool) {
    error!(
        context = context,
        error = %err,
        http_status = ?err.http_status(),
        error_code = ?err.error_code(),
        retriable,
        payload = ?err.payload(),
        "Pixoo interaction failed"
    );
}

fn client_timeout() -> Duration {
    env::var("PIXOO_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map_or_else(|| Duration::from_secs(10), Duration::from_millis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::routing::post;
    use axum::Router;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::net::TcpListener;

    fn with_retry_policy(
        mut client: PixooClient,
        retries: usize,
        backoff: Duration,
    ) -> PixooClient {
        client.retries = retries;
        client.backoff = backoff;
        client
    }

    #[derive(Clone)]
    struct SequenceState {
        statuses: Arc<Vec<StatusCode>>,
        counter: Arc<AtomicUsize>,
    }

    async fn sequence_handler(State(state): State<SequenceState>) -> (StatusCode, String) {
        let index = state.counter.fetch_add(1, Ordering::SeqCst);
        let status = state
            .statuses
            .get(index)
            .copied()
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, r#"{"error_code":0}"#.to_string())
    }

    async fn start_sequence_server(statuses: Vec<StatusCode>) -> (String, Arc<AtomicUsize>) {
        let counter = Arc::new(AtomicUsize::new(0));
        let state = SequenceState {
            statuses: Arc::new(statuses),
            counter: counter.clone(),
        };
        let app = Router::new()
            .route("/post", post(sequence_handler))
            .with_state(state);
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server");
        });

        (format!("http://{addr}/post"), counter)
    }

    #[test]
    fn builds_payload_with_command_and_args() {
        let mut args = Map::new();
        args.insert("Minute".to_string(), Value::Number(1.into()));
        args.insert("Second".to_string(), Value::Number(0.into()));
        args.insert("Status".to_string(), Value::Number(1.into()));

        let payload = PixooClient::build_payload(&PixooCommand::SystemReboot, args);

        assert_eq!(
            payload.get("Command"),
            Some(&Value::String("Device/SysReboot".to_string()))
        );
        assert_eq!(payload.get("Minute"), Some(&Value::Number(1.into())));
        assert_eq!(payload.get("Second"), Some(&Value::Number(0.into())));
        assert_eq!(payload.get("Status"), Some(&Value::Number(1.into())));
    }

    #[test]
    fn parses_success_response_without_error_code() {
        let response = parse_response(
            &json!({
                "error_code": 0,
                "Brightness": 100,
                "RotationFlag": 1
            })
            .to_string(),
        )
        .expect("response should parse");

        assert!(response.get("error_code").is_none());
        assert_eq!(response.get("Brightness"), Some(&Value::Number(100.into())));
        assert_eq!(response.get("RotationFlag"), Some(&Value::Number(1.into())));
    }

    #[test]
    fn returns_error_on_device_failure() {
        let err = parse_response(&json!({ "error_code": 12 }).to_string())
            .expect_err("expected device error");

        match err {
            PixooError::DeviceError { code, .. } => assert_eq!(code, 12),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn rejects_invalid_responses() {
        let err = parse_response("not-json").expect_err("expected invalid response");
        assert!(matches!(err, PixooError::InvalidResponse(_)));

        let err = parse_response(&json!({ "Status": 1 }).to_string())
            .expect_err("expected missing error_code");
        assert!(matches!(err, PixooError::MissingErrorCode));

        let err = parse_response(&json!(true).to_string()).expect_err("expected non-object error");
        assert!(matches!(err, PixooError::InvalidResponse(_)));
    }

    #[test]
    fn parses_error_code_from_strings() {
        let response = parse_response(&json!({ "error_code": "0" }).to_string())
            .expect("string error_code should parse");
        assert!(response.is_empty());

        let err = parse_response(&json!({ "error_code": "abc" }).to_string())
            .expect_err("expected invalid error_code");
        assert!(matches!(err, PixooError::InvalidErrorCode(_)));
    }

    #[tokio::test]
    async fn returns_http_status_error_on_failure() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(POST).path("/post");
            then.status(503).body(r#"{"error_code":0}"#);
        });

        let client = with_retry_policy(
            PixooClient::new(server.base_url()).expect("client"),
            0,
            Duration::from_millis(10),
        );
        let err = client
            .send_command(PixooCommand::SystemReboot, Map::new())
            .await
            .expect_err("expected http status error");

        assert!(matches!(err, PixooError::HttpStatus(503)));
        mock.assert();
    }

    #[tokio::test]
    async fn retries_on_server_errors_until_success() {
        let (base_url, counter) = start_sequence_server(vec![
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::OK,
        ])
        .await;

        let client = PixooClient::new(base_url).expect("client");
        let response = client
            .send_command(PixooCommand::SystemReboot, Map::new())
            .await
            .expect("request should succeed");

        assert!(response.is_empty());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn does_not_retry_on_client_errors() {
        let (base_url, counter) = start_sequence_server(vec![StatusCode::BAD_REQUEST]).await;

        let client = PixooClient::new(base_url).expect("client");
        let err = client
            .send_command(PixooCommand::SystemReboot, Map::new())
            .await
            .expect_err("expected http status error");

        assert!(matches!(err, PixooError::HttpStatus(400)));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    #[allow(clippy::type_complexity)]
    async fn backoff_increments_between_retries() {
        // Track timestamps when each request arrives to verify backoff delays.
        let timestamps = Arc::new(std::sync::Mutex::new(Vec::<std::time::Instant>::new()));
        let timestamps_clone = timestamps.clone();

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let statuses = Arc::new(vec![
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::OK,
        ]);

        let state = (statuses, counter_clone, timestamps_clone);
        let app = Router::new()
            .route(
                "/post",
                post(
                    |State((statuses, counter, timestamps)): State<(
                        Arc<Vec<StatusCode>>,
                        Arc<AtomicUsize>,
                        Arc<std::sync::Mutex<Vec<std::time::Instant>>>,
                    )>| async move {
                        timestamps.lock().unwrap().push(std::time::Instant::now());
                        let index = counter.fetch_add(1, Ordering::SeqCst);
                        let status = statuses
                            .get(index)
                            .copied()
                            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                        (status, r#"{"error_code":0}"#.to_string())
                    },
                ),
            )
            .with_state(state);

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server");
        });

        let base_url = format!("http://{addr}/post");

        // Use short backoff delays for fast testing.
        let client = with_retry_policy(
            PixooClient::new(base_url).expect("client"),
            2,
            Duration::from_millis(50),
        );

        let response = client
            .send_command(PixooCommand::SystemReboot, Map::new())
            .await
            .expect("request should succeed");

        assert!(response.is_empty());
        assert_eq!(counter.load(Ordering::SeqCst), 3);

        // Verify backoff delays increased between retries.
        let ts = timestamps.lock().unwrap();
        assert_eq!(ts.len(), 3);

        let first_delay = ts[1].duration_since(ts[0]);
        let second_delay = ts[2].duration_since(ts[1]);

        // First backoff should be ~50ms, second should be ~100ms (doubled).
        // Allow some tolerance for scheduling variance.
        assert!(
            first_delay >= Duration::from_millis(40),
            "first delay {first_delay:?} should be >= 40ms"
        );
        assert!(
            second_delay >= Duration::from_millis(80),
            "second delay {second_delay:?} should be >= 80ms"
        );
        assert!(
            second_delay > first_delay,
            "second delay {second_delay:?} should be > first delay {first_delay:?}"
        );
    }

    #[tokio::test]
    async fn sends_post_with_json_content_type() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/post")
                .header("content-type", "application/json");
            then.status(200)
                .header("content-type", "text/html")
                .body(r#"{"error_code":0}"#);
        });

        let client = PixooClient::new(server.base_url()).expect("client");
        let response = client
            .send_command(PixooCommand::SystemReboot, Map::new())
            .await
            .expect("request should succeed");

        assert!(response.is_empty());
        mock.assert();
    }
}
