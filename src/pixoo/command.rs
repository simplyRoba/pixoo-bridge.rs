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
