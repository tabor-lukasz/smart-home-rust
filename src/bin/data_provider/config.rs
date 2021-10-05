
use std::{net::SocketAddr, path::Path};

use serde::Deserialize;

#[derive(Debug,Clone,Deserialize)]
pub struct Config {
    pub location: i32,
    pub db_addr: SocketAddr,
    pub db_user: String,
    pub db_pass: String,
}

impl Config {
    pub fn new(path: &Path) -> Self {

        let serialized = std::fs::read_to_string(path).unwrap();
        let rval: Config = serde_yaml::from_str(&serialized).unwrap();

        rval
    }
}