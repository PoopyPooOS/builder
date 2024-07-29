use crate::{
    parser,
    types::{Component, ComponentType, Config},
};
use std::io;

pub fn build(config: &Config, _iso: bool) -> io::Result<()> {
    let components = parser::read_components(&config.components_dir)?;

    for component in components {
        match component.component_type {
            ComponentType::Binary(_) => build_binary(component),
            ComponentType::Module => build_module(component),
            ComponentType::Other => Ok(()),
        }?
    }

    Ok(())
}

pub fn build_binary(component: Component) -> io::Result<()> {
    println!("Building binary: {}", component.name);

    Ok(())
}

pub fn build_module(module: Component) -> io::Result<()> {
    let module_components = parser::read_components(&module.path)?;

    println!("{:#?}", module_components);

    Ok(())
}
