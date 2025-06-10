
pub async fn health_check() -> &'static str {
    "live"
}

pub async fn reboot() -> &'static str {
    "reboot"
}