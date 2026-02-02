pub mod manage;
pub mod system;
pub mod tools;

pub use manage::mount_manage_routes;
pub use system::mount_system_routes;
pub use tools::mount_tool_routes;
