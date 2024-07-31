use crate::{
    parser,
    types::{Component, ComponentType, Config},
};
use rayon::prelude::*;
use std::io;

pub fn build(config: &Config, _iso: bool) -> io::Result<()> {
    let components = parser::read_components(&config.components_dir)?;

    components
        .par_iter()
        .filter(|component| component.component_type == ComponentType::Binary)
        .for_each(|component| build_component(component).expect("Failed to compile component"));

    Ok(())
}

fn build_component(component: &Component) -> io::Result<()> {
    println!("Building binary: {}", component.name);

    Ok(())
}
