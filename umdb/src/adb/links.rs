use std::process::Command;

use crate::{core::Configuration, common::links::{OpenDeepLinkError, OpenDeepLinkResult}};

pub fn adb_open_deep_link(configuration: &Configuration, device_id: &str, link: &str) -> Result<OpenDeepLinkResult, OpenDeepLinkError> {
    let adb_command = configuration
    .adb_command
    .as_deref()
    .ok_or(OpenDeepLinkError::DebugBridgePathMissing)?;

    let output = Command
    ::new(adb_command)
    .args(["-s", device_id, "shell", "am", "start", "-W", "-a", "android.intent.action.VIEW", "-d", link])
    .output()
    .map_err(|error| OpenDeepLinkError::CannotRunProcess(error.to_string()))?;

    if !output.status.success() {
        return Err(OpenDeepLinkError::BadExitCode(output.status.code()));
    }

    let output = String::from_utf8_lossy(&output.stdout);

    let new_instance_started = !output.contains("Activity not started, intent has been delivered to currently running top-most instance.");

    let lines = output.trim_end().split("\n").collect::<Vec<&str>>();

    let launched = *lines.last().unwrap() == "Complete";

    if launched {
        return Ok(if new_instance_started { OpenDeepLinkResult::Started } else { OpenDeepLinkResult::LaunchedInExistingInstance });
    }

    Err(OpenDeepLinkError::CommandFailed(
        lines
        .iter()
        .find(|line| line.starts_with("Error: "))
        .unwrap_or(&"")
        .to_string()
    ))
}
