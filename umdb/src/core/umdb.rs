use super::configuration::Configuration;

#[derive(PartialEq)]
pub enum System {
    Android,
    Ios,
}

pub struct Umdb {
    pub configuration: Configuration,
    pub enable_logs:   bool,
}

impl Umdb {
    pub fn new() -> Umdb {
        Umdb { configuration: Configuration::new(), enable_logs: true }
    }
}
