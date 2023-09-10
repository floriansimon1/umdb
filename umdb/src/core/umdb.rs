use super::configuration::Configuration;

pub struct Umdb {
    pub configuration: Configuration,
    pub enable_logs:   bool,
}

impl Umdb {
    pub fn new() -> Umdb {
        Umdb { configuration: Configuration::new(), enable_logs: true }
    }
}
