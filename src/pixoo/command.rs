use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum PixooCommand {
    DeviceSysReboot,
}

impl PixooCommand {
    pub fn as_str(&self) -> &str {
        match self {
            PixooCommand::DeviceSysReboot => "Device/SysReboot",
        }
    }
}

impl fmt::Display for PixooCommand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_sys_reboot_command() {
        let command = PixooCommand::DeviceSysReboot;
        assert_eq!(command.as_str(), "Device/SysReboot");
        assert_eq!(command.to_string(), "Device/SysReboot");
    }
}
