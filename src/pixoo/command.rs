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
    ManageSetLocation,
    ManageSetTimezone,
    ManageSetUtc,
    ManageSetTimeMode,
    ManageSetTemperatureUnit,
    ManageDisplayPower,
    ManageDisplayBrightness,
    ManageDisplayRotation,
    ManageDisplayMirror,
    ManageDisplayOverclock,
    ManageDisplayWhiteBalance,
    DrawGetGifId,
    DrawSendGif,
    #[allow(dead_code)] // Required by spec but not yet used
    DrawResetGifId,
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
            PixooCommand::ManageSetLocation => "Sys/LogAndLat",
            PixooCommand::ManageSetTimezone => "Sys/TimeZone",
            PixooCommand::ManageSetUtc => "Device/SetUTC",
            PixooCommand::ManageSetTimeMode => "Device/SetTime24Flag",
            PixooCommand::ManageSetTemperatureUnit => "Device/SetDisTempMode",
            PixooCommand::ManageDisplayPower => "Channel/OnOffScreen",
            PixooCommand::ManageDisplayBrightness => "Channel/SetBrightness",
            PixooCommand::ManageDisplayRotation => "Device/SetScreenRotationAngle",
            PixooCommand::ManageDisplayMirror => "Device/SetMirrorMode",
            PixooCommand::ManageDisplayOverclock => "Device/SetHighLightMode",
            PixooCommand::ManageDisplayWhiteBalance => "Device/SetWhiteBalance",
            PixooCommand::DrawGetGifId => "Draw/GetHttpGifId",
            PixooCommand::DrawSendGif => "Draw/SendHttpGif",
            PixooCommand::DrawResetGifId => "Draw/ResetHttpGifId",
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
        assert_eq!(PixooCommand::ManageSetLocation.as_str(), "Sys/LogAndLat");
        assert_eq!(PixooCommand::ManageSetTimezone.as_str(), "Sys/TimeZone");
        assert_eq!(PixooCommand::ManageSetUtc.as_str(), "Device/SetUTC");
        assert_eq!(
            PixooCommand::ManageSetTimeMode.as_str(),
            "Device/SetTime24Flag"
        );
        assert_eq!(
            PixooCommand::ManageSetTemperatureUnit.as_str(),
            "Device/SetDisTempMode"
        );
        assert_eq!(
            PixooCommand::ManageDisplayPower.as_str(),
            "Channel/OnOffScreen"
        );
        assert_eq!(
            PixooCommand::ManageDisplayBrightness.as_str(),
            "Channel/SetBrightness"
        );
        assert_eq!(
            PixooCommand::ManageDisplayRotation.as_str(),
            "Device/SetScreenRotationAngle"
        );
        assert_eq!(
            PixooCommand::ManageDisplayMirror.as_str(),
            "Device/SetMirrorMode"
        );
        assert_eq!(
            PixooCommand::ManageDisplayOverclock.as_str(),
            "Device/SetHighLightMode"
        );
        assert_eq!(
            PixooCommand::ManageDisplayWhiteBalance.as_str(),
            "Device/SetWhiteBalance"
        );
        assert_eq!(PixooCommand::DrawGetGifId.as_str(), "Draw/GetHttpGifId");
        assert_eq!(PixooCommand::DrawSendGif.as_str(), "Draw/SendHttpGif");
        assert_eq!(PixooCommand::DrawResetGifId.as_str(), "Draw/ResetHttpGifId");
    }
}
