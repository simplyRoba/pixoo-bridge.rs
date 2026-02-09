use crate::pixoo::PixooClient;

#[derive(Clone)]
pub struct AppState {
    pub health_forward: bool,
    pub pixoo_client: PixooClient,
    pub animation_speed_factor: f64,
    pub max_image_size: usize,
}

#[cfg(test)]
impl AppState {
    pub fn with_client(client: PixooClient) -> Self {
        Self {
            health_forward: false,
            pixoo_client: client,
            animation_speed_factor: 1.4,
            max_image_size: 5 * 1024 * 1024,
        }
    }
}
