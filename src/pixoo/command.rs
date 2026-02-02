use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum PixooCommand {
    SystemReboot,
    ToolsTimer,
    ToolsStopwatch,
    ToolsScoreboard,
    ToolsSoundMeter,
    ManageGetSettings,
    ManageGetTime,
    ManageGetWeather,
}

impl PixooCommand {
    pub fn as_str(&self) -> &str {
        match self {
            PixooCommand::SystemReboot => "Device/SysReboot",
            PixooCommand::ToolsTimer => "Tools/SetTimer",
            PixooCommand::ToolsStopwatch => "Tools/SetStopWatch",
            PixooCommand::ToolsScoreboard => "Tools/SetScoreBoard",
            PixooCommand::ToolsSoundMeter => "Tools/SetNoiseStatus",
            PixooCommand::ManageGetSettings => "Channel/GetAllConf",
            PixooCommand::ManageGetTime => "Device/GetDeviceTime",
            PixooCommand::ManageGetWeather => "Device/GetWeatherInfo",
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
    fn string_values_remain_unchanged() {
        assert_eq!(PixooCommand::SystemReboot.as_str(), "Device/SysReboot");
        assert_eq!(PixooCommand::ToolsTimer.as_str(), "Tools/SetTimer");
        assert_eq!(PixooCommand::ToolsStopwatch.as_str(), "Tools/SetStopWatch");
        assert_eq!(
            PixooCommand::ToolsScoreboard.as_str(),
            "Tools/SetScoreBoard"
        );
        assert_eq!(
            PixooCommand::ToolsSoundMeter.as_str(),
            "Tools/SetNoiseStatus"
        );
        assert_eq!(
            PixooCommand::ManageGetSettings.as_str(),
            "Channel/GetAllConf"
        );
        assert_eq!(PixooCommand::ManageGetTime.as_str(), "Device/GetDeviceTime");
        assert_eq!(
            PixooCommand::ManageGetWeather.as_str(),
            "Device/GetWeatherInfo"
        );
    }
}
