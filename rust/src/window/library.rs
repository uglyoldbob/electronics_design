//! The schematic window is used to view and manipulate a library of components and footprints.

use egui_multiwin::egui_glow::EguiGlow;
use egui_multiwin::{
    egui,
    multi_window::NewWindowRequest,
    tracked_window::{RedrawResponse, TrackedWindow},
};

use crate::library::LibraryAction;
use crate::symbol::MouseMode;
use crate::symbol::SymbolWidgetSelection;
use crate::MyApp;

/// The window structure
pub struct Library {
    /// Set when the title needs to change.
    new_title: Option<String>,
    /// The name of the library selected for viewing / editing
    selected_library: Option<String>,
    /// The symbol selected for viewing / editing
    selected_symbol: Option<String>,
    /// The selected objects for the symbol being modified
    selection: Vec<crate::symbol::SymbolWidgetSelection>,
    /// Used to indicate that there are changes to library changes
    old_saved_status: bool,
    /// The mouse mode for the library editor.
    mm: MouseMode,
}

impl Library {
    /// Create a new window
    pub fn request() -> NewWindowRequest<MyApp> {
        NewWindowRequest {
            window_state: Box::new(Self {
                new_title: None,
                selected_library: None,
                selected_symbol: None,
                selection: Vec::new(),
                old_saved_status: false,
                mm: MouseMode::Selection,
            }),
            builder: egui_multiwin::glutin::window::WindowBuilder::new()
                .with_resizable(true)
                .with_inner_size(egui_multiwin::glutin::dpi::LogicalSize {
                    width: 800.0,
                    height: 600.0,
                })
                .with_title(format!("{} Library Editor", crate::PACKAGE_NAME)),
            options: egui_multiwin::tracked_window::TrackedWindowOptions {
                vsync: false,
                shader: None,
            },
        }
    }
}

impl TrackedWindow for Library {
    type Data = MyApp;

    fn is_root(&self) -> bool {
        true
    }

    fn set_root(&mut self, _root: bool) {}

    fn opengl_after(
        &mut self,
        _c: &mut Self::Data,
        gl_window: &mut egui_multiwin::glutin::WindowedContext<
            egui_multiwin::glutin::PossiblyCurrent,
        >,
    ) {
        if let Some(title) = self.new_title.take() {
            gl_window.window().set_title(&title);
        }
    }

    fn redraw(&mut self, c: &mut MyApp, egui: &mut EguiGlow) -> RedrawResponse<Self::Data> {
        let mut quit = false;

        let mut windows_to_create = vec![];

        let is_saved = c.library_log.is_saved();
        if self.old_saved_status != is_saved {
            self.old_saved_status = is_saved;
            self.new_title = if is_saved {
                Some(format!("{} Library Editor", crate::PACKAGE_NAME.to_string()))
            }
            else 
            {
                Some(format!("*{} Library Editor", crate::PACKAGE_NAME.to_string()))
            };
        }

        egui::TopBottomPanel::top("menubar").show(&egui.egui_ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.menu_button("File", |ui| {
                    if let Some(lib) = &self.selected_library {
                        if ui.button("Save all libraries").clicked() {
                            let mut no_errors = true;
                            for (_libname, lib) in &mut c.libraries {
                                if let Some(libh) = lib {
                                    if libh.can_save() {
                                        if let Err(e) = libh.save() {
                                            no_errors = false;
                                            let _e = native_dialog::MessageDialog::new()
                                                .set_type(native_dialog::MessageType::Error)
                                                .set_title(&format!(
                                                    "Failed to save {} library",
                                                    libh.library.name
                                                ))
                                                .set_text(e.to_string().as_str())
                                                .show_alert();
                                        }
                                    } else {
                                        no_errors = false;
                                        let _e = native_dialog::MessageDialog::new()
                                            .set_type(native_dialog::MessageType::Error)
                                            .set_title("Failed to save library")
                                            .set_text("No path to save library exists?")
                                            .show_alert();
                                    }
                                }
                            }
                            if no_errors {
                                c.library_log.set_saved(true);
                            }
                            ui.close_menu();
                        }
                    }
                    ui.menu_button("Recent", |ui| {
                        if ui.button("Thing 1").clicked() {
                            ui.close_menu();
                        }
                    });
                    if ui.button("Quit").clicked() {
                        quit = true;
                        ui.close_menu();
                    }
                });
                let (undoable, redoable) = (c.library_log.can_undo(), c.library_log.can_redo());
                ui.menu_button("Edit", |ui| {
                    if ui
                        .add_enabled(undoable, egui::Button::new("Undo"))
                        .clicked()
                    {
                        c.library_log.undo(&mut c.libraries);
                        ui.close_menu();
                    }
                    if ui
                        .add_enabled(redoable, egui::Button::new("Redo"))
                        .clicked()
                    {
                        c.library_log.redo(&mut c.libraries);
                        ui.close_menu();
                    }
                });
            });
        });

        let mut input = egui.egui_ctx.input_mut();
        if input.consume_shortcut(&egui::KeyboardShortcut {
            modifiers: egui::Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
                mac_cmd: false,
                command: false,
            },
            key: egui::Key::Z,
        }) {
            if c.library_log.can_undo() {
                c.library_log.undo(&mut c.libraries);
            }
        }
        if input.consume_shortcut(&egui::KeyboardShortcut {
            modifiers: egui::Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
                mac_cmd: false,
                command: false,
            },
            key: egui::Key::Y,
        }) {
            if c.library_log.can_redo() {
                c.library_log.redo(&mut c.libraries);
            }
        }
        drop(input);

        egui::TopBottomPanel::top("button bar").show(&egui.egui_ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.mm, MouseMode::Selection, "S")
                    .on_hover_ui(|ui| {
                        ui.label("Selection mode");
                    });
                ui.selectable_value(&mut self.mm, MouseMode::TextDrag, "T")
                    .on_hover_ui(|ui| {
                        ui.label("Text move mode");
                    });
                ui.selectable_value(&mut self.mm, MouseMode::NewText, "t")
                    .on_hover_ui(|ui| {
                        ui.label("Create Text mode");
                    });
            });
        });

        egui::SidePanel::left("left panel")
            .resizable(true)
            .show(&egui.egui_ctx, |ui| {
                egui::TopBottomPanel::top("library select")
                    .resizable(true)
                    .show_inside(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .always_show_scroll(true)
                            .auto_shrink([false, false])
                            .stick_to_right(true)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    if ui.button("New library").clicked() {
                                        windows_to_create
                                            .push(super::library_name::LibraryName::request());
                                    }
                                    if let Some(l) = &self.selected_library {
                                        if ui.button("Delete Library").clicked() {
                                            c.library_log.apply(
                                                &mut c.libraries,
                                                crate::library::LibraryAction::DeleteLibrary {
                                                    name: l.clone(),
                                                    old_lib: None,
                                                },
                                            );
                                            self.selected_library = None;
                                        }
                                    }
                                });
                                ui.separator();
                                for name in c.libraries.keys() {
                                    if ui
                                        .selectable_label(
                                            self.selected_library == Some(name.clone()),
                                            name.clone(),
                                        )
                                        .clicked()
                                    {
                                        self.selected_library = Some(name.clone());
                                    }
                                }
                            });
                    });

                let mut actionlog = Vec::new();

                if let Some(l) = &self.selected_library {
                    let check = c.libraries.get_mut(l);
                    if let Some(Some(lib)) = check {
                        egui::TopBottomPanel::top("symbol select")
                            .resizable(true)
                            .show_inside(ui, |ui| {
                                egui::ScrollArea::vertical()
                                    .id_source("symbol scroll")
                                    .always_show_scroll(true)
                                    .auto_shrink([false, false])
                                    .stick_to_right(true)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            if ui.button("New symbol").clicked() {
                                                windows_to_create.push(
                                                    crate::window::symbol_name::SymbolName::request(
                                                        l.clone(),
                                                    ),
                                                );
                                            }
                                            if let Some(symname) = &self.selected_symbol {
                                                if ui.button("Delete Symbol").clicked() {
                                                    actionlog.push(LibraryAction::DeleteSymbol {
                                                        libname: l.clone(),
                                                        symname: symname.clone(),
                                                        symbol: None,
                                                    });
                                                    self.selected_symbol = None;
                                                }
                                            }
                                        });
                                        ui.separator();
                                        for name in lib.library.syms.keys() {
                                            if ui
                                                .selectable_label(
                                                    self.selected_symbol == Some(name.clone()),
                                                    name,
                                                )
                                                .clicked()
                                            {
                                                self.selected_symbol = Some(name.clone());
                                            }
                                        }
                                    });
                            });
                    }
                }

                for a in actionlog {
                    c.library_log.apply(&mut c.libraries, a);
                }
            });

        let mut actionlog = Vec::new();

        if let Some(l) = &self.selected_library {
            let check = c.libraries.get_mut(l);
            if let Some(Some(lib)) = check {
                if let Some(sym) = &self.selected_symbol {
                    egui::SidePanel::right("right panel").resizable(true).show(
                        &egui.egui_ctx,
                        |ui| {
                            egui::ScrollArea::vertical()
                                .always_show_scroll(true)
                                .auto_shrink([false, false])
                                .stick_to_right(true)
                                .show(ui, |ui| {
                                    if self.selection.len() == 1 {
                                        ui.label("There is a selection");
                                        let sel = &self.selection[0];
                                        let symbol = &lib.library.syms[sym];
                                        match sel {
                                            SymbolWidgetSelection::Text { textnum } => {
                                                let t = &symbol.texts[*textnum];
                                                ui.label("Text Properties");
                                                let mut text = t.text.clone();
                                                ui.horizontal(|ui| {
                                                    ui.label("Text ");
                                                    ui.add(egui::TextEdit::singleline(&mut text));
                                                });
                                                if text != t.text {
                                                    actionlog.push(LibraryAction::EditText {
                                                        libname: l.clone(),
                                                        symname: sym.clone(),
                                                        textnum: *textnum,
                                                        old: t.text.clone(),
                                                        new: text,
                                                    });
                                                }
                                                let mut xstr = format!("{:.4}", t.x);
                                                ui.horizontal(|ui| {
                                                    ui.label("X ");
                                                    ui.add(egui::TextEdit::singleline(&mut xstr));
                                                });
                                                if let Ok(x) = xstr.parse::<f32>() {
                                                    if (x - t.x) > f32::EPSILON {
                                                        actionlog.push(LibraryAction::MoveText {
                                                            libname: l.clone(),
                                                            symname: sym.clone(),
                                                            textnum: *textnum,
                                                            dx: x - t.x,
                                                            dy: 0.0,
                                                        });
                                                    }
                                                }
                                                let mut ystr = format!("{:.4}", t.y);
                                                ui.horizontal(|ui| {
                                                    ui.label("Y ");
                                                    ui.add(egui::TextEdit::singleline(&mut ystr));
                                                });
                                                if let Ok(y) = ystr.parse::<f32>() {
                                                    if (y - t.y) > f32::EPSILON {
                                                        actionlog.push(LibraryAction::MoveText {
                                                            libname: l.clone(),
                                                            symname: sym.clone(),
                                                            textnum: *textnum,
                                                            dx: 0.0,
                                                            dy: y - t.y,
                                                        });
                                                    }
                                                }
                                                let mut color = t.color();
                                                if ui.color_edit_button_srgba(&mut color).changed()
                                                {
                                                    actionlog.push(
                                                        LibraryAction::ChangeTextColor {
                                                            libname: l.clone(),
                                                            symname: sym.clone(),
                                                            textnum: *textnum,
                                                            old: t.color(),
                                                            new: color,
                                                        },
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    ui.label("Right");
                                });
                        },
                    );
                }
            }
        }

        for a in actionlog {
            c.library_log.apply(&mut c.libraries, a);
        }

        let mut actions = Vec::new();
        egui::CentralPanel::default().show(&egui.egui_ctx, |ui| {
            self.selected_library.as_ref().and_then(|l| {
                c.libraries.get_mut(l).and_then(|a| {
                    a.as_mut().and_then(|lh| {
                        self.selected_symbol.as_ref().map(|symname| {
                            let lib = &mut lh.library;
                            if let Some(sym) = lib.syms.get_mut(symname) {
                                let mut sym =
                                    crate::symbol::SymbolDefinitionHolder::new(sym, l.clone());
                                let sym = crate::symbol::SymbolDefinitionWidget::new(
                                    &mut sym,
                                    &mut self.mm,
                                    &mut self.selection,
                                    &mut actions,
                                );
                                ui.add(sym);
                            }
                        })
                    })
                })
            });
        });
        for a in actions {
            c.library_log.apply(&mut c.libraries, a);
        }

        RedrawResponse {
            quit,
            new_windows: windows_to_create,
        }
    }
}
