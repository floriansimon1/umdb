use serde::Serialize;

#[derive(Serialize)]
pub enum OpenDeepLinkResult {
    Started,
    LaunchedInExistingInstance,
}

#[derive(Serialize)]
pub enum OpenDeepLinkError {
    CannotRunProcess(String),
    BadExitCode(Option<i32>),
    DebugBridgePathMissing,
    CommandFailed(String),
}
