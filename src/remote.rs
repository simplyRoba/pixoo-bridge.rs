use reqwest::{header::CONTENT_TYPE, Client, StatusCode};
use std::fmt;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct RemoteFetchConfig {
    pub timeout: Duration,
    pub max_image_size: usize,
}

impl RemoteFetchConfig {
    pub fn new(timeout: Duration, max_image_size: usize) -> Self {
        Self {
            timeout,
            max_image_size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemoteAsset {
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
}

#[derive(Debug)]
pub enum RemoteFetchError {
    RequestFailed(reqwest::Error),
    Status(StatusCode),
    TooLarge { limit: usize, actual: usize },
}

impl fmt::Display for RemoteFetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RemoteFetchError::RequestFailed(err) => write!(f, "{err}"),
            RemoteFetchError::Status(status) => {
                write!(f, "remote server responded with status {status}")
            }
            RemoteFetchError::TooLarge { limit, actual } => {
                write!(f, "remote payload exceeds limit {limit} (actual {actual})")
            }
        }
    }
}

#[derive(Clone)]
pub struct RemoteFetcher {
    client: Client,
    max_image_size: usize,
}

impl RemoteFetcher {
    pub fn new(config: RemoteFetchConfig) -> Result<Self, reqwest::Error> {
        let client = Client::builder().timeout(config.timeout).build()?;
        Ok(Self {
            client,
            max_image_size: config.max_image_size,
        })
    }

    pub async fn fetch(&self, link: &str) -> Result<RemoteAsset, RemoteFetchError> {
        let response = self
            .client
            .get(link)
            .send()
            .await
            .map_err(RemoteFetchError::RequestFailed)?;

        let status = response.status();
        if !status.is_success() {
            return Err(RemoteFetchError::Status(status));
        }

        if let Some(length) = response.content_length() {
            let length = usize::try_from(length).unwrap_or(usize::MAX);
            if length > self.max_image_size {
                return Err(RemoteFetchError::TooLarge {
                    limit: self.max_image_size,
                    actual: length,
                });
            }
        }

        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.split(';').next().unwrap_or(value).trim().to_string());

        let mut body: Vec<u8> = Vec::new();
        let mut response = response;
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(RemoteFetchError::RequestFailed)?
        {
            if body.len() + chunk.len() > self.max_image_size {
                return Err(RemoteFetchError::TooLarge {
                    limit: self.max_image_size,
                    actual: body.len() + chunk.len(),
                });
            }
            body.extend_from_slice(&chunk);
        }

        Ok(RemoteAsset {
            bytes: body,
            content_type,
        })
    }
}
