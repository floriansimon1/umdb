use std::{fs, os::unix::prelude::PermissionsExt};

use pathsearch::find_executable_in_path;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum CheckExecutableError {
    ProcessExecutionError(String),
    BadExitCode(Option<i32>),
    CheckFileError(String),
    CannotCheckVersion,
    NotAnExecutable,
    NotAFile,
}

pub fn perform_common_executable_checks(path: &str) -> Result<String, CheckExecutableError> {
    if let Some(path) = find_executable_in_path(path) {
        return Ok(path.into_os_string().into_string().unwrap());
    }

    let file_metadata = fs::metadata(path).map_err(|error| {
        CheckExecutableError::CheckFileError(error.to_string())
    })?;

    if !file_metadata.is_file() {
        return Err(CheckExecutableError::NotAFile);
    }

    if cfg!(not(target_os = "windows")) && file_metadata.permissions().mode() & 0o111 == 0 {
        return Err(CheckExecutableError::NotAnExecutable);
    }

    Ok(path.to_string())
}
