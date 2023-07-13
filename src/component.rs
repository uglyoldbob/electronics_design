//! This module defines what a component is. It brings together the schematic symbol, and footprint.

use std::collections::HashMap;

use crate::library::LibraryHolder;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[non_exhaustive]
/// A reference to a variant of a component
pub struct ComponentVariantReference {
    ///The library the symbol belongs to
    pub lib: String,
    /// The commponent name in the library
    pub com: String,
    /// The variant name of the component
    pub var: String,
    /// The position of the variant
    pub pos: crate::general::Coordinates,
}

impl ComponentVariantReference {
    pub fn get_component<'a>(
        &self,
        libs: &'a HashMap<String, LibraryHolder>,
    ) -> Option<&'a ComponentVariant> {
        let mut ret = None;
        if let Some(libh) = libs.get(&self.lib) {
            if let Some(lib) = &libh.library {
                if let Some(component) = lib.components.get(&self.com) {
                    ret = component.variants.get(&self.var);
                }
            }
        }
        ret
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[non_exhaustive]
/// A variant of a component, with links to one schematic symbol and one or more pcb footprints that might apply to it
pub struct ComponentVariant {
    /// The symbol for the variant
    pub symbol: Option<crate::symbol::SymbolReference>,
    /// The name of the variant
    pub name: String,
}

impl ComponentVariant {
    /// Create a named component variant
    pub fn new(name: String) -> Self {
        Self { symbol: None, name }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
/// The definition of a component
pub struct ComponentDefinition {
    /// The name of the component
    name: String,
    /// The variants of a component are intended to be somewhat interchangable with each other.
    pub variants: HashMap<String, ComponentVariant>,
}

impl ComponentDefinition {
    /// Create a blank symbol with the given name
    pub fn new(name: String) -> Self {
        Self {
            name,
            variants: HashMap::new(),
        }
    }
}
