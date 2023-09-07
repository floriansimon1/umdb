use super::configuration::Configuration;

pub struct Umdb {
    pub configuration: Configuration,
}

impl Umdb {
    pub fn new() -> Umdb {
        Umdb { configuration: Configuration::new() }
    } 
}
