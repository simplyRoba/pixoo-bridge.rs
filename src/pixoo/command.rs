use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum PixooCommand {
    ChannelSetCloudIndex,
    ToolsSetTimer,
    Raw(String),
}

impl PixooCommand {
    pub fn as_str(&self) -> &str {
        match self {
            PixooCommand::ChannelSetCloudIndex => "Channel/SetCloudIndex",
            PixooCommand::ToolsSetTimer => "Tools/SetTimer",
            PixooCommand::Raw(command) => command,
        }
    }
}

impl fmt::Display for PixooCommand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<&str> for PixooCommand {
    fn from(value: &str) -> Self {
        PixooCommand::Raw(value.to_string())
    }
}

impl From<String> for PixooCommand {
    fn from(value: String) -> Self {
        PixooCommand::Raw(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_command_passthrough() {
        let command = PixooCommand::Raw("Custom/Command".to_string());
        assert_eq!(command.as_str(), "Custom/Command");
        assert_eq!(command.to_string(), "Custom/Command");
    }

    #[test]
    fn from_str_and_string_create_raw() {
        let from_str: PixooCommand = "Custom/Command".into();
        let from_string: PixooCommand = "Custom/Command".to_string().into();

        assert_eq!(from_str, PixooCommand::Raw("Custom/Command".to_string()));
        assert_eq!(from_string, PixooCommand::Raw("Custom/Command".to_string()));
    }
}
