use std::process::Command;

use crate::{common::device::{Device, DeviceListingError}, core::Configuration};

pub fn adb_devices(configuration: &Configuration) -> Result<Vec<Device>, DeviceListingError> {
    let adb_command = configuration
    .adb_command
    .as_deref()
    .ok_or(DeviceListingError::DebugBridgePathMissing)?;

    let output = Command
    ::new(adb_command)
    .arg("devices")
    .output()
    .map_err(|error| DeviceListingError::CannotRunProcess(error.to_string()))?;

    if !output.status.success() {
        return Err(DeviceListingError::BadExitCode(output.status.code()));
    }

    Ok(
        String
        ::from_utf8_lossy(&output.stdout)
        .trim_end()
        .split("\n")
        .skip(1)
        .map(parse_line)
        .collect()
    )
}

fn parse_line(line: &str) -> Device {
    Device { id: line.split("\t").next().unwrap().to_string() }
}

