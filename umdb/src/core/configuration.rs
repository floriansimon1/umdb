use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Configuration {
    pub adb_command: Option<String>,
}

impl Configuration {
    pub fn new() -> Configuration {
        Configuration { adb_command: None }
    }
}
