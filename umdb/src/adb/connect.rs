use std::{net::IpAddr, time::Duration, process::ExitStatus};

use serde::Serialize;
use tokio::{process::Command, time::timeout};

use crate::core::Configuration;

#[derive(Serialize)]
pub enum AdbConnectError {
    CannotRunProcess(String),
    DebugBridgePathMissing,
    CannotConnectToDevice,
    CannotSwitchAdbMode,
    DeviceUnresponsive,
}

pub async fn adb_connect(configuration: &Configuration, device_id: &str, ip: &IpAddr, port: u16) -> Result<(), AdbConnectError> {
    let adb_command = configuration
    .adb_command
    .as_deref()
    .ok_or(AdbConnectError::DebugBridgePathMissing)?;

    let process_task = Command
    ::new(adb_command)
    .args(&["-s", device_id, "tcpip", &format!("{port}")])
    .output();

    let result = timeout(Duration::from_secs(1), process_task).await;

    let output = match result {
        Ok(Ok(output)) => output,
        Err(_)         => return Err(AdbConnectError::DeviceUnresponsive),
        Ok(Err(error)) => return Err(AdbConnectError::CannotRunProcess(error.to_string())),
    };

    if !ExitStatus::success(&output.status) {
        return Err(AdbConnectError::CannotSwitchAdbMode)
    }

    tokio::time::sleep(Duration::from_secs(1)).await;

    let process_task = Command
    ::new(adb_command)
    .args(&["connect", &format!("{ip}:{port}")])
    .output();

    let result = timeout(Duration::from_secs(1), process_task).await;

    match result {
        Ok(Ok(_))      => {},
        Err(_)         => return Err(AdbConnectError::DeviceUnresponsive),
        Ok(Err(error)) => return Err(AdbConnectError::CannotRunProcess(error.to_string())),
    };

    if !ExitStatus::success(&output.status) {
        return Err(AdbConnectError::CannotConnectToDevice)
    }

    return Ok(());
}
