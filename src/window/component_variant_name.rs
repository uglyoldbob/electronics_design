//! This window asks the user for a name of the new library

use egui_multiwin::egui_glow::EguiGlow;
use egui_multiwin::egui;
use crate::egui_multiwin_dynamic::{
    multi_window::NewWindowRequest,
    tracked_window::{RedrawResponse, TrackedWindow},
};

use crate::library::LibraryAction;
use crate::MyApp;

/// The window structure
pub struct Name {
    /// The name of the library being modified
    lib_name: String,
    /// The name of the component being modified
    component_name: String,
    /// The name of the new symbol
    name: String,
}

impl Name {
    /// Create a new window
    pub fn request(lib_name: String, component_name: String) -> NewWindowRequest {
        NewWindowRequest::new(
            super::Windows::ComponentVariantName(Self {
                lib_name,
                component_name,
                name: "".to_string(),
            }),
            egui_multiwin::winit::window::WindowBuilder::new()
                .with_resizable(true)
                .with_inner_size(egui_multiwin::winit::dpi::LogicalSize {
                    width: 320.0,
                    height: 240.0,
                })
                .with_title("New Component Variant"),
            egui_multiwin::tracked_window::TrackedWindowOptions {
                vsync: false,
                shader: None,
            },
            egui_multiwin::multi_window::new_id(),
        )
    }
}

impl TrackedWindow for Name {
    fn is_root(&self) -> bool {
        false
    }

    fn set_root(&mut self, _root: bool) {}

    fn redraw(
        &mut self,
        c: &mut MyApp,
        egui: &mut EguiGlow,
        _window: &egui_multiwin::winit::window::Window,
        _clipboard: &mut egui_multiwin::arboard::Clipboard,
    ) -> RedrawResponse {
        let mut quit = false;

        let windows_to_create = vec![];

        let mut actionlog = Vec::new();

        egui::CentralPanel::default().show(&egui.egui_ctx, |ui| {
            ui.label("Please enter a name for the new component variant");
            let te = egui::widgets::TextEdit::singleline(&mut self.name)
                .hint_text("Component variant name");
            ui.add(te).request_focus();
            let lib = c.libraries.get(&self.lib_name);
            if let Some(lib) = lib {
                if let Some(library) = &lib.library {
                    if let Some(component) = library.components.get(&self.component_name) {
                        if !self.name.is_empty() && component.variants.contains_key(&self.name) {
                            ui.colored_label(
                                egui::Color32::RED,
                                "Component variant already exists",
                            );
                        } else if ui.button("Create").clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            if !library.syms.contains_key(&self.name) {
                                actionlog.push(LibraryAction::CreateComponentVariant {
                                    libname: self.lib_name.clone(),
                                    comname: self.component_name.clone(),
                                    varname: self.name.clone(),
                                    variant: None,
                                });
                            }
                            quit = true;
                        }
                    } else {
                        ui.label("Component does not exist for some reason");
                    }
                }
            } else {
                ui.label("Library does not exist for some reason");
            }
        });

        for a in actionlog {
            c.library_log.apply(&mut c.libraries, a);
        }

        RedrawResponse {
            quit,
            new_windows: windows_to_create,
        }
    }
}
