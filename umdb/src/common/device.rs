use serde::Serialize;

#[derive(Serialize)]
pub enum DeviceListingError {
    UnrecognizedDebugBridgeOutput,
    CannotRunProcess(String),
    BadExitCode(Option<i32>),
    DebugBridgePathMissing,
}

#[derive(Serialize)]
pub struct Device {
    pub id: String,
    pub is_remote: bool,
    pub is_offline: bool,
    pub model: Option<String>,
    pub alias: Option<String>,
}
