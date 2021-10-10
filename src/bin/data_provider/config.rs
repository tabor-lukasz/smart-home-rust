
use std::{net::SocketAddr, path::Path};

use serde::Deserialize;

#[derive(Debug,Clone,Deserialize)]
pub struct Config {
    pub sensor_hw_id: String,
    pub db_addr: SocketAddr,
    pub db_user: String,
    pub db_pass: String,
    pub db_name: String,
}

impl Config {
    pub fn new(path: &Path) -> Self {

        let serialized = std::fs::read_to_string(path).unwrap();
        let rval: Config = serde_yaml::from_str(&serialized).unwrap();

        rval
    }
}