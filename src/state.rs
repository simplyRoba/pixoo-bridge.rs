use pixoo_bridge::pixoo::PixooClient;

#[derive(Clone)]
pub struct AppState {
    pub health_forward: bool,
    pub pixoo_client: Option<PixooClient>,
}
