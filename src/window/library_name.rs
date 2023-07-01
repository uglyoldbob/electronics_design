//! This window asks the user for a name of the new library

use egui_multiwin::egui_glow::EguiGlow;
use egui_multiwin::{
    egui,
    multi_window::NewWindowRequest,
    tracked_window::{RedrawResponse, TrackedWindow},
};
use strum::IntoEnumIterator;

use crate::MyApp;

/// Defines messages that can some from other threads
enum Message {
    ///A library is being created and saved to the specified location
    CreateNewLibrary(crate::general::StoragePath),
}

/// The window structure
pub struct LibraryName {
    /// The name of the library being created
    name: String,
    /// For the dropdown
    selected_path: crate::general::StoragePath,
    /// The message channel for communicating with the main thread, when needed.
    message_channel: (
        std::sync::mpsc::Sender<Message>,
        std::sync::mpsc::Receiver<Message>,
    ),
}

impl LibraryName {
    /// Create a new window
    pub fn request() -> NewWindowRequest<MyApp> {
        NewWindowRequest {
            window_state: Box::new(Self {
                name: "".to_string(),
                selected_path: crate::general::StoragePath::default(),
                message_channel: std::sync::mpsc::channel(),
            }),
            builder: egui_multiwin::winit::window::WindowBuilder::new()
                .with_resizable(true)
                .with_inner_size(egui_multiwin::winit::dpi::LogicalSize {
                    width: 320.0,
                    height: 240.0,
                })
                .with_title("New Library"),
            options: egui_multiwin::tracked_window::TrackedWindowOptions {
                vsync: false,
                shader: None,
            },
        }
    }
}

impl TrackedWindow<MyApp> for LibraryName {
    fn is_root(&self) -> bool {
        false
    }

    fn set_root(&mut self, _root: bool) {}

    fn redraw(
        &mut self,
        c: &mut MyApp,
        egui: &mut EguiGlow,
        _window: &egui_multiwin::winit::window::Window,
    ) -> RedrawResponse<MyApp> {
        let mut quit = false;

        let windows_to_create = vec![];

        while let Ok(message) = self.message_channel.1.try_recv() {
            match message {
                Message::CreateNewLibrary(p) => {
                    if !c.libraries.contains_key(&self.name) {
                        c.library_log.apply(
                            &mut c.libraries,
                            crate::library::LibraryAction::CreateNewLibrary {
                                name: self.name.clone(),
                                lib: None,
                            },
                        );
                    }
                    if let Some(Some(lib)) = c.libraries.get_mut(&self.name) {
                        lib.set_path(p);
                        if let Err(e) = lib.save() {
                            let _e = native_dialog::MessageDialog::new()
                                .set_type(native_dialog::MessageType::Error)
                                .set_title("Failed to save library")
                                .set_text(e.to_string().as_str())
                                .show_alert();
                        }
                    }
                    quit = true;
                }
            }
        }

        egui::CentralPanel::default().show(&egui.egui_ctx, |ui| {
            ui.label("Please enter a name for the new library");
            let te = egui::widgets::TextEdit::singleline(&mut self.name).hint_text("Library name");
            ui.add(te).request_focus();
            if !self.name.is_empty() && c.libraries.contains_key(&self.name) {
                ui.colored_label(egui::Color32::RED, "Library already exists");
            } else {
                egui::ComboBox::from_label("Select one!")
                    .selected_text(self.selected_path.display())
                    .show_ui(ui, |ui| {
                        for saveoption in crate::general::StoragePath::iter() {
                            if ui.selectable_label(false, saveoption.display()).clicked() {
                                self.selected_path = saveoption;
                            }
                        }
                    });
                match &self.selected_path {
                    crate::general::StoragePath::LocalFilesystem(_p) => {
                        if ui.button("Select save path").clicked() {
                            let f = rfd::AsyncFileDialog::new()
                                .add_filter("Raw", &["uol"])
                                .set_title("Save library")
                                .set_directory(directories::ProjectDirs::data_dir(
                                    c.dirs.as_ref().unwrap(),
                                ))
                                .save_file();
                            let message_sender = self.message_channel.0.clone();
                            crate::execute(async move {
                                let file = f.await;
                                if let Some(file) = file {
                                    let mut fname = file.path().to_path_buf();
                                    fname.set_extension("uol");
                                    message_sender
                                        .send(Message::CreateNewLibrary(
                                            crate::general::StoragePath::LocalFilesystem(
                                                fname.into_os_string().into_string().unwrap(),
                                            ),
                                        ))
                                        .ok();
                                }
                            });
                        }
                    }
                }
            }
        });

        RedrawResponse {
            quit,
            new_windows: windows_to_create,
        }
    }
}
