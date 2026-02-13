use crate::pixoo::PixooClient;
#[cfg(test)]
use crate::remote::RemoteFetchConfig;
use crate::remote::RemoteFetcher;
#[cfg(test)]
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub health_forward: bool,
    pub pixoo_client: PixooClient,
    pub animation_speed_factor: f64,
    pub max_image_size: usize,
    pub remote_fetcher: RemoteFetcher,
}

#[cfg(test)]
impl AppState {
    pub fn with_client(client: PixooClient) -> Self {
        let remote_fetcher = RemoteFetcher::new(RemoteFetchConfig::new(
            Duration::from_millis(10_000),
            5 * 1024 * 1024,
        ))
        .expect("remote fetcher");
        Self {
            health_forward: false,
            pixoo_client: client,
            animation_speed_factor: 1.4,
            max_image_size: 5 * 1024 * 1024,
            remote_fetcher,
        }
    }
}
