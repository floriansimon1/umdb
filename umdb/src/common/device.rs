use serde::Serialize;

#[derive(Serialize)]
pub enum DeviceListingError {
    CannotRunProcess(String),
    BadExitCode(Option<i32>),
    DebugBridgePathMissing,
}

#[derive(Serialize)]
pub struct Device {
    pub id: String,
}
