use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    name: String,
    descrtiption: String,
}

#[allow(dead_code)]
pub fn parse(module_path: &Path) -> io::Result<Config> {
    let module_config_raw = fs::read_to_string(module_path.join("module.toml"))?;

    Ok(toml::from_str(&module_config_raw).expect("Failed to parse module config"))
}
