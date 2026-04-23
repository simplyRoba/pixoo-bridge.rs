// Pixoo protocol field-name constants.
//
// Using constants instead of raw string literals ensures that typos
// are caught at compile time and makes it easy to search for all
// usages of a given protocol field.

/// Field names inserted into outgoing command payloads.
pub mod request {
    // ── System / command envelope ──
    pub const COMMAND: &str = "Command";

    // ── Manage: location ──
    pub const LONGITUDE: &str = "Longitude";
    pub const LATITUDE: &str = "Latitude";

    // ── Manage: clock ──
    pub const UTC: &str = "Utc";
    pub const TIMEZONE_VALUE: &str = "TimeZoneValue";

    // ── Manage: display ──
    pub const ON_OFF: &str = "OnOff";
    pub const BRIGHTNESS: &str = "Brightness";
    pub const MODE: &str = "Mode";

    // ── Manage: white balance ──
    pub const R_VALUE: &str = "RValue";
    pub const G_VALUE: &str = "GValue";
    pub const B_VALUE: &str = "BValue";

    // ── Tools: timer ──
    pub const MINUTE: &str = "Minute";
    pub const SECOND: &str = "Second";
    pub const STATUS: &str = "Status";

    // ── Tools: scoreboard ──
    pub const BLUE_SCORE: &str = "BlueScore";
    pub const RED_SCORE: &str = "RedScore";

    // ── Tools: sound meter ──
    pub const NOISE_STATUS: &str = "NoiseStatus";

    // ── Draw: text ──
    pub const LCD_ID: &str = "LcdId";
    pub const TEXT_ID: &str = "TextId";
    pub const X: &str = "x";
    pub const Y: &str = "y";
    pub const DIR: &str = "dir";
    pub const FONT: &str = "font";
    pub const TEXT_WIDTH: &str = "TextWidth";
    pub const SPEED: &str = "speed";
    pub const TEXT_STRING: &str = "TextString";
    pub const COLOR: &str = "color";
    pub const ALIGN: &str = "align";

    // ── Draw: gif / frame ──
    pub const PIC_ID: &str = "PicId";
    pub const PIC_NUM: &str = "PicNum";
    pub const PIC_OFFSET: &str = "PicOffset";
    pub const PIC_WIDTH: &str = "PicWidth";
    pub const PIC_SPEED: &str = "PicSpeed";
    pub const PIC_DATA: &str = "PicData";
}

/// Field names read from device response payloads.
pub mod response {
    // ── Manage: settings ──
    pub const LIGHT_SWITCH: &str = "LightSwitch";
    pub const BRIGHTNESS: &str = "Brightness";
    pub const MIRROR_FLAG: &str = "MirrorFlag";
    pub const CUR_CLOCK_ID: &str = "CurClockId";
    pub const TIME_24_FLAG: &str = "Time24Flag";
    pub const ROTATION_FLAG: &str = "RotationFlag";
    pub const TEMPERATURE_MODE: &str = "TemperatureMode";

    // ── Manage: time ──
    pub const UTC_TIME: &str = "UTCTime";
    pub const LOCAL_TIME: &str = "LocalTime";

    // ── Manage: weather ──
    pub const WEATHER: &str = "Weather";
    pub const CUR_TEMP: &str = "CurTemp";
    pub const MIN_TEMP: &str = "MinTemp";
    pub const MAX_TEMP: &str = "MaxTemp";
    pub const PRESSURE: &str = "Pressure";
    pub const HUMIDITY: &str = "Humidity";
    pub const WIND_SPEED: &str = "WindSpeed";

    // ── Draw ──
    pub const PIC_ID: &str = "PicId";
}
