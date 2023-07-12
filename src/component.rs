//! This module defines what a component is. It brings together the schematic symbol, and footprint.

use std::collections::HashMap;

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
