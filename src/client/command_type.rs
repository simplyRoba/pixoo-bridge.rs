pub enum CommandType {
    // System
    Reboot,
    // Sound
    PlaySound,
    // Draw
    GetNextPictureId,
    ResetPictureId,
    DrawAnimation,
    DrawText,
    ClearText,
    DrawCommandList,
    // Clocks
    SetClock,
    GetClock,
    // Manage
    SetDisplayOnOff,
    SetDisplayBrightness,
    // ... and much more
    // Tool
    
}

// impl CommandType {
//     pub const fn command_value(&self) -> &'static str {
//         match self { 
//             CommandType::Reboot => "Device/SysReboot",
//             CommandType::PlaySound() => "Device/PlayBuzzer",
//             CommandType::GET_NEXT_PICTURE_ID() => "Draw/GetHttpGifId",
//         }
//     }
// }
