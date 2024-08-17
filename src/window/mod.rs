//! Defines the various window types used by the program

pub mod component_name;
pub mod component_variant_name;
pub mod library;
pub mod library_name;
pub mod schematic;
pub mod symbol_name;

use egui_multiwin::egui_glow::EguiGlow;
use std::sync::Arc;
use egui_multiwin::enum_dispatch::enum_dispatch;
use crate::egui_multiwin_dynamic::tracked_window::{RedrawResponse, TrackedWindow};
use crate::ipc;

/// The windows for the program
#[enum_dispatch(TrackedWindow)]
pub enum Windows {
    /// The component name window
    ComponentName(component_name::Name),
    /// The component variant name window
    ComponentVariantName(component_variant_name::Name),
    /// Library name window
    LibraryName(library_name::LibraryName),
    /// The library window
    Library(library::Library),
    /// The schematic window
    Schematic(schematic::SchematicWindow),
    /// The symbol name window
    SymbolName(symbol_name::SymbolName),
}