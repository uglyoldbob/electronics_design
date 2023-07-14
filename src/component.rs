//! This module defines what a component is. It brings together the schematic symbol, and footprint.

use std::collections::HashMap;

use crate::{
    library::{Library, LibraryHolder},
    symbol::SymbolDefinition,
};

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
    /// Get the componentvariant that the reference refers to
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

    /// Get a SymbolDefinition from the componentvariantreference.
    pub fn get_symbol<'a>(
        &self,
        libs: &'a HashMap<String, LibraryHolder>,
    ) -> Option<&'a SymbolDefinition> {
        let mut ret = None;
        if let Some(component) = self.get_component(libs) {
            if let Some(sym) = &component.symbol {
                if let Some(libraryh) = libs.get(&self.lib) {
                    if let Some(library) = &libraryh.library {
                        if let Some(libh) = libs.get(&sym.lib.get_name(library)) {
                            if let Some(lib) = &libh.library {
                                ret = lib.syms.get(&sym.sym);
                            }
                        }
                    }
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
