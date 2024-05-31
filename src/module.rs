use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ModuleConfig {
    name: String,
    descrtiption: String,
}

#[allow(dead_code)]
pub fn parse_module(module_path: PathBuf) -> io::Result<ModuleConfig> {
    let module_config_raw = fs::read_to_string(module_path.join("module.toml"))?;

    Ok(toml::from_str(&module_config_raw).expect("Failed to parse module config"))
}
