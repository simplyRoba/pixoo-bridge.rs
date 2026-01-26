use crate::pixoo::command::PixooCommand;
use crate::pixoo::error::PixooError;
use reqwest::header::CONTENT_TYPE;
use serde_json::{Map, Value};
use std::time::Duration;
use tokio::time::sleep;

pub type PixooResponse = Map<String, Value>;

#[derive(Debug, Clone)]
pub struct PixooClient {
    base_url: String,
    http: reqwest::Client,
    retries: usize,
    backoff: Duration,
}

impl PixooClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self, PixooError> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self {
            base_url: base_url.into(),
            http,
            retries: 2,
            backoff: Duration::from_millis(200),
        })
    }

    pub fn from_ip(ip: impl Into<String>) -> Result<Self, PixooError> {
        let ip = ip.into();
        let base_url = format!("http://{ip}/post");
        Self::new(base_url)
    }

    pub fn with_retry_policy(mut self, retries: usize, backoff: Duration) -> Self {
        self.retries = retries;
        self.backoff = backoff;
        self
    }

    pub fn build_payload(
        command: &PixooCommand,
        mut args: Map<String, Value>,
    ) -> Map<String, Value> {
        args.insert(
            "Command".to_string(),
            Value::String(command.as_str().to_string()),
        );
        args
    }

    pub async fn send_command(
        &self,
        command: PixooCommand,
        args: Map<String, Value>,
    ) -> Result<PixooResponse, PixooError> {
        let payload = Self::build_payload(&command, args);
        self.execute_with_retry(&payload).await
    }

    async fn execute_with_retry(
        &self,
        payload: &Map<String, Value>,
    ) -> Result<PixooResponse, PixooError> {
        let mut attempt = 0;

        loop {
            match self.execute_once(payload).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    if attempt >= self.retries || !is_retriable(&err) {
                        return Err(err);
                    }

                    attempt += 1;
                    let delay = self.backoff * attempt as u32;
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
            .post(&self.base_url)
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
            .or_else(|| number.as_u64().map(|value| value as i64))
            .ok_or_else(|| PixooError::InvalidErrorCode(value.clone())),
        Value::String(text) => text
            .parse::<i64>()
            .map_err(|_| PixooError::InvalidErrorCode(value.clone())),
        _ => Err(PixooError::InvalidErrorCode(value.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use serde_json::json;

    #[test]
    fn builds_payload_with_command_and_args() {
        let mut args = Map::new();
        args.insert("Minute".to_string(), Value::Number(1.into()));
        args.insert("Second".to_string(), Value::Number(0.into()));
        args.insert("Status".to_string(), Value::Number(1.into()));

        let payload = PixooClient::build_payload(&PixooCommand::ToolsSetTimer, args);

        assert_eq!(
            payload.get("Command"),
            Some(&Value::String("Tools/SetTimer".to_string()))
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

        let client = PixooClient::new(server.url("/post")).expect("client");
        let response = client
            .send_command(PixooCommand::ChannelSetCloudIndex, Map::new())
            .await
            .expect("request should succeed");

        assert!(response.is_empty());
        mock.assert();
    }
}
