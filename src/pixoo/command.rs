use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum PixooCommand {
    DeviceSysReboot,
    ToolsSetTimer,
    ToolsSetStopWatch,
    ToolsSetScoreBoard,
    ToolsSetNoiseStatus,
}

impl PixooCommand {
    pub fn as_str(&self) -> &str {
        match self {
            PixooCommand::DeviceSysReboot => "Device/SysReboot",
            PixooCommand::ToolsSetTimer => "Tools/SetTimer",
            PixooCommand::ToolsSetStopWatch => "Tools/SetStopWatch",
            PixooCommand::ToolsSetScoreBoard => "Tools/SetScoreBoard",
            PixooCommand::ToolsSetNoiseStatus => "Tools/SetNoiseStatus",
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

    #[test]
    fn tools_commands_have_expected_strings() {
        assert_eq!(PixooCommand::ToolsSetTimer.as_str(), "Tools/SetTimer");
        assert_eq!(
            PixooCommand::ToolsSetStopWatch.as_str(),
            "Tools/SetStopWatch"
        );
        assert_eq!(
            PixooCommand::ToolsSetScoreBoard.as_str(),
            "Tools/SetScoreBoard"
        );
        assert_eq!(
            PixooCommand::ToolsSetNoiseStatus.as_str(),
            "Tools/SetNoiseStatus"
        );
    }
}
