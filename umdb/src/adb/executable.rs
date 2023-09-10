use std::process::Command;

use crate::common::executable::{CheckExecutableError, perform_common_executable_checks};

pub fn check_adb(path: &str) -> Result<(), CheckExecutableError> {
    let path = perform_common_executable_checks(path)?;

    let output = Command
    ::new(path)
    .arg("--version")
    .output()
    .map_err(|error| CheckExecutableError::ProcessExecutionError(error.to_string()))?;

    if !output.status.success() {
        return Err(CheckExecutableError::BadExitCode(output.status.code()));
    }

    if !String::from_utf8_lossy(&output.stdout).starts_with("Android Debug Bridge") {
        return Err(CheckExecutableError::CannotCheckVersion);
    }

    Ok(())
}

