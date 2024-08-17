//! Defines things used for ipc

/// Messages that can be sent between processes
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[non_exhaustive]
pub enum IpcMessage {
    /// Create a new library editor
    NewLibrary,
    /// Create a new schematic window
    NewSchematic,
}

use egui_multiwin::winit::window::WindowId;

impl IpcMessage {
    pub fn window_id(&self) -> Option<WindowId> {
        None
    }
}