//! The schematic window is used to view and manipulate a library of components and footprints.

use egui_multiwin::egui::Sense;
use egui_multiwin::egui_glow::EguiGlow;
use egui_multiwin::{
    egui,
    multi_window::NewWindowRequest,
    tracked_window::{RedrawResponse, TrackedWindow},
};

use crate::component::ComponentVariant;
use crate::library::LibraryAction;
use crate::symbol::SymbolWidgetSelection;
use crate::symbol::{LibraryReference, MouseMode};
use crate::MyApp;

/// An enumeration of things that be selected in the library editor
#[derive(PartialEq)]
enum Thing {
    /// A symbol has been selected
    Symbol(String),
    /// A component has been selected
    Component(String),
}

/// The window structure
pub struct Library {
    /// The name of the library selected for viewing / editing
    selected_library: Option<String>,
    /// The thing selected for viewing / editing
    selected_thing: Option<Thing>,
    /// The selected variant of a component
    selected_variant: Option<String>,
    /// The selected library when editing a component variant
    selected_variant_library: Option<String>,
    /// The selected objects for the symbol being modified
    selection: Vec<crate::symbol::SymbolWidgetSelection>,
    /// Used to indicate that there are changes to library changes
    old_saved_status: bool,
    /// The mouse mode for the library editor.
    mm: MouseMode,
    /// The origin for the symbol drawing
    origin: crate::general::Coordinates,
    /// True when the widget should recenter
    recenter: bool,
    /// The zoom factor for the widget
    zoom: f32,
    /// The angle for new pins, in degrees
    pin_angle: f32,
}

impl Library {
    /// Create a new window
    pub fn request() -> NewWindowRequest<MyApp> {
        NewWindowRequest {
            window_state: Box::new(Self {
                selected_library: None,
                selected_thing: None,
                selected_variant: None,
                selected_variant_library: None,
                selection: Vec::new(),
                old_saved_status: false,
                mm: MouseMode::Selection,
                origin: crate::general::Coordinates::Inches(0.0, 0.0),
                recenter: false,
                zoom: 115.0,
                pin_angle: 0.0,
            }),
            builder: egui_multiwin::winit::window::WindowBuilder::new()
                .with_resizable(true)
                .with_inner_size(egui_multiwin::winit::dpi::LogicalSize {
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

impl TrackedWindow<MyApp> for Library {
    fn is_root(&self) -> bool {
        true
    }

    fn set_root(&mut self, _root: bool) {}

    fn redraw(
        &mut self,
        c: &mut MyApp,
        egui: &mut EguiGlow,
        window: &egui_multiwin::winit::window::Window,
    ) -> RedrawResponse<MyApp> {
        let mut quit = false;

        let mut windows_to_create = vec![];

        let is_saved = c.library_log.is_saved();
        if self.old_saved_status != is_saved {
            self.old_saved_status = is_saved;
            let new_title = if is_saved {
                format!("{} Library Editor", crate::PACKAGE_NAME)
            } else {
                format!("*{} Library Editor", crate::PACKAGE_NAME)
            };
            window.set_title(&new_title);
        }

        egui::TopBottomPanel::top("menubar").show(&egui.egui_ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save all libraries").clicked() {
                        let mut no_errors = true;
                        for libh in &mut c.libraries.values() {
                            if let Some(library) = &libh.library {
                                if libh.can_save() {
                                    if let Err(e) = libh.save() {
                                        no_errors = false;
                                        let _e = native_dialog::MessageDialog::new()
                                            .set_type(native_dialog::MessageType::Error)
                                            .set_title(&format!(
                                                "Failed to save {} library",
                                                library.name
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

        let input = egui.egui_ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut {
                modifiers: egui::Modifiers {
                    alt: false,
                    ctrl: true,
                    shift: false,
                    mac_cmd: false,
                    command: false,
                },
                key: egui::Key::Z,
            })
        });
        if input && c.library_log.can_undo() {
            c.library_log.undo(&mut c.libraries);
        }
        let input = egui.egui_ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut {
                modifiers: egui::Modifiers {
                    alt: false,
                    ctrl: true,
                    shift: false,
                    mac_cmd: false,
                    command: false,
                },
                key: egui::Key::Y,
            })
        });
        if input && c.library_log.can_redo() {
            c.library_log.redo(&mut c.libraries);
        }

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
                ui.selectable_value(&mut self.mm, MouseMode::NewPin, "P")
                    .on_hover_ui(|ui| {
                        ui.label("Create pin");
                    });
            });
        });

        egui::SidePanel::left("left panel")
            .resizable(true)
            .show(&egui.egui_ctx, |ui| {
                egui::TopBottomPanel::top("library select")
                    .resizable(true)
                    .show_inside(ui, |ui| {
                        egui::CollapsingHeader::new("Libraries")
                            .default_open(true)
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical()
                                    .scroll_bar_visibility(
                                        egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                                    )
                                    .auto_shrink([false, false])
                                    .stick_to_right(true)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            if ui.button("New library").clicked() {
                                                windows_to_create.push(
                                                    super::library_name::LibraryName::request(),
                                                );
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
                                        for (name, lib) in &c.libraries {
                                            let response = ui
                                            .selectable_label(
                                                self.selected_library == Some(name.clone()),
                                                name.clone(),
                                            );
                                            if response.clicked()
                                            {
                                                self.selected_library = Some(name.clone());
                                            }
                                            response.context_menu(|ui| {
                                                let mut haspath = false;
                                                    if let Some(lh) = c.libraries.get(name) {
                                                        if let Some(path) = &lh.path {
                                                            let path = path.open_path();
                                                            if let Some(mut path) = path {
                                                                haspath = true;
                                                                if ui.add(egui::Label::new("Goto").sense(Sense::click())).clicked() {
                                                                    path.pop();
                                                                    open::that_in_background(path);
                                                                    ui.close_menu();
                                                                }
                                                            }
                                                        }
                                                }
                                                if !haspath {
                                                    ui.label("Cannot browse to");
                                                }
                                            });
                                        }
                                    });
                            });
                    });

                let mut actionlog = Vec::new();

                if let Some(l) = &self.selected_library {
                    let check = c.libraries.get_mut(l);
                    if let Some(lib) = check {
                        if let Some(library) = &lib.library {
                        egui::TopBottomPanel::top("symbol select")
                            .resizable(true)
                            .show_inside(ui, |ui| {
                                egui::CollapsingHeader::new("Symbols")
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        egui::ScrollArea::vertical()
                                    .id_source("symbol scroll")
                                    .scroll_bar_visibility(
                                        egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                                    )
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
                                            if let Some(Thing::Symbol(symname)) = &self.selected_thing {
                                                if ui.button("Delete Symbol").clicked() {
                                                    actionlog.push(LibraryAction::DeleteSymbol {
                                                        libname: l.clone(),
                                                        symname: symname.clone(),
                                                        symbol: None,
                                                    });
                                                    self.selected_thing = None;
                                                }
                                            }
                                        });
                                        ui.separator();
                                        for name in library.syms.keys() {
                                            if ui
                                                .selectable_label(
                                                    self.selected_thing == Some(Thing::Symbol(name.clone())),
                                                    name,
                                                )
                                                .clicked()
                                            {
                                                self.selected_thing = Some(Thing::Symbol(name.clone()));
                                                self.recenter = true;
                                            }
                                        }
                                    });
                                    });
                                ui.separator();
                                egui::CollapsingHeader::new("Components")
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        egui::ScrollArea::vertical()
                                    .id_source("component scroll")
                                    .scroll_bar_visibility(
                                        egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                                    )
                                    .auto_shrink([false, false])
                                    .stick_to_right(true)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            if ui.button("New component").clicked() {
                                                windows_to_create.push(
                                                    crate::window::component_name::Name::request(
                                                        l.clone(),
                                                    ),
                                                );
                                            }
                                            if let Some(Thing::Component(comname)) = &self.selected_thing {
                                                if ui.button("Delete Component").clicked() {
                                                    actionlog.push(LibraryAction::DeleteComponent {
                                                        libname: l.clone(),
                                                        comname: comname.clone(),
                                                        component: None,
                                                    });
                                                    self.selected_thing = None;
                                                }
                                            }
                                        });
                                        ui.separator();
                                        for name in library.components.keys() {
                                            if ui
                                                .selectable_label(
                                                    self.selected_thing == Some(Thing::Component(name.clone())),
                                                    name,
                                                )
                                                .clicked()
                                            {
                                                self.selected_thing = Some(Thing::Component(name.clone()));
                                                self.recenter = true;
                                            }
                                        }
                                    });
                                    });
                            });
                        }
                    }
                }

                for a in actionlog {
                    c.library_log.apply(&mut c.libraries, a);
                }
            });

        let mut actionlog = Vec::new();

        if let Some(l) = &self.selected_library {
            let check = c.libraries.get_mut(l);
            if let Some(lib) = check {
                if let Some(library) = &lib.library {
                    if let Some(Thing::Symbol(sym)) = &self.selected_thing {
                        egui::SidePanel::right("right panel").resizable(true).show(
                        &egui.egui_ctx,
                        |ui| {
                            egui::ScrollArea::vertical()
                                .scroll_bar_visibility(
                                    egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                                )
                                .auto_shrink([false, false])
                                .stick_to_right(true)
                                .show(ui, |ui| {
                                    match self.selection.len() {
                                        0 => {}
                                        1 => {
                                            let sel = &self.selection[0];
                                            let symbol = &library.syms[sym];
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
                                                    let units = t.location.get_units(c.units);
                                                    let mut xstr = format!("{:.4}", units.0);
                                                    ui.horizontal(|ui| {
                                                        ui.label("X ");
                                                        ui.add(egui::TextEdit::singleline(&mut xstr));
                                                    });
                                                    if let Ok(x) = xstr.parse::<f32>() {
                                                        if t.location.changed_x(x) {
                                                            actionlog.push(LibraryAction::MoveText {
                                                                libname: l.clone(),
                                                                symname: sym.clone(),
                                                                textnum: *textnum,
                                                                delta: crate::general::Coordinates::from_pos2(egui::pos2(x - units.0, 0.0), 1.0),
                                                            });
                                                        }
                                                    }
                                                    let mut ystr = format!("{:.4}", units.1);
                                                    ui.horizontal(|ui| {
                                                        ui.label("Y ");
                                                        ui.add(egui::TextEdit::singleline(&mut ystr));
                                                    });
                                                    if let Ok(y) = ystr.parse::<f32>() {
                                                        if t.location.changed_y(y) {
                                                            actionlog.push(LibraryAction::MoveText {
                                                                libname: l.clone(),
                                                                symname: sym.clone(),
                                                                textnum: *textnum,
                                                                delta: crate::general::Coordinates::from_pos2(egui::pos2(0.0, units.1 - y), 1.0),
                                                            });
                                                        }
                                                    }
                                                    let mut color = t.color.get_color32(crate::general::ColorMode::ScreenModeDark);
                                                    if ui.color_edit_button_srgba(&mut color).changed()
                                                    {
                                                        actionlog.push(
                                                            LibraryAction::ChangeTextColor {
                                                                libname: l.clone(),
                                                                symname: sym.clone(),
                                                                textnum: *textnum,
                                                                old: t.color,
                                                                new: crate::schematic::Colors::Custom(color.to_srgba_unmultiplied()),
                                                            },
                                                        );
                                                    }
                                                }
                                                SymbolWidgetSelection::Pin { pinnum } => {
                                                    if symbol.pins.len() >= (pinnum + 1) {
                                                        let p = &symbol.pins[*pinnum];
                                                        ui.label("Pin has properties");
                                                    }
                                                }
                                            }
                                        }
                                        _ => {
                                            ui.label("There are multiple selections");
                                        }
                                    }
                                    ui.label("Right");
                                });
                        },
                    );
                    }
                }
            }
        }

        for a in actionlog {
            c.library_log.apply(&mut c.libraries, a);
        }

        let mut component_modify = None;

        let mut actions = Vec::new();
        let mut component_changed = false;
        egui::CentralPanel::default().show(&egui.egui_ctx, |ui| {
            self.selected_library.as_ref().and_then(|l| {
                c.libraries.get_mut(l).and_then(|lh| {
                    self.selected_thing.as_ref().map(|thing| {
                        let lib = &lh.library;
                        if let Some(library) = lib {
                            match thing {
                                Thing::Symbol(symname) => {
                                    if let Some(sym) = library.syms.get(symname) {
                                        let mut sym = crate::symbol::SymbolDefinitionHolder::new(
                                            sym,
                                            l.clone(),
                                        );
                                        let sym = crate::symbol::SymbolDefinitionWidget::new(
                                            &mut sym,
                                            &mut self.mm,
                                            &mut self.selection,
                                            &mut actions,
                                            &mut self.origin,
                                            &mut self.zoom,
                                            self.recenter,
                                            &mut self.pin_angle,
                                        );
                                        self.recenter = false;
                                        let resp = ui.add(sym);
                                        if resp.dragged_by(egui::PointerButton::Middle) {
                                            self.origin += crate::general::Coordinates::from_pos2(
                                                resp.drag_delta().to_pos2(),
                                                self.zoom,
                                            );
                                        }
                                        if resp.double_clicked_by(egui::PointerButton::Middle) {
                                            self.recenter = true;
                                        }
                                        if resp.hovered() {
                                            let scroll = ui.input(|i| i.scroll_delta);
                                            if scroll.y.abs() > f32::EPSILON {
                                                self.zoom *= f32::powf(1.0025, scroll.y);
                                            }
                                        }
                                    }
                                }
                                Thing::Component(comname) => {
                                    if let Some(com) = library.components.get(comname) {
                                        let mut cb = egui::ComboBox::from_label("Select variant");
                                        if let Some(selvar) = &self.selected_variant {
                                            if let Some(var) = com.variants.get(selvar) {
                                                component_modify = Some(var.to_owned());
                                                cb = cb.selected_text(&var.name);
                                            }
                                        }
                                        cb.show_ui(ui, |ui| {
                                            for (name, var) in com.variants.iter() {
                                                if ui.selectable_label(false, &var.name).clicked() {
                                                    self.selected_variant = Some(name.clone());
                                                }
                                            }
                                        });
                                        if ui.button("Add variant").clicked() {
                                            windows_to_create.push(
                                            crate::window::component_variant_name::Name::request(
                                                l.clone(),
                                                comname.clone(),
                                            ),
                                        );
                                        }
                                        if let Some(selvar) = &self.selected_variant {
                                            if ui.button("Delete variant").clicked() {
                                                actions.push(
                                                    LibraryAction::DeleteComponentVariant {
                                                        libname: l.clone(),
                                                        comname: comname.clone(),
                                                        varname: selvar.clone(),
                                                        variant: None,
                                                    },
                                                );
                                            }
                                            ui.separator();
                                        }
                                    }
                                }
                            }
                        }
                    })
                })
            });
            let mut cb = egui::ComboBox::from_label("Select library for symbol");
            if let Some(l) = &self.selected_variant_library {
                cb = cb.selected_text(l.clone());
            }
            cb.show_ui(ui, |ui| {
                for l in c.libraries.keys() {
                    if ui.selectable_label(false, l).clicked() {
                        self.selected_variant_library = Some(l.clone());
                    }
                }
            });
            if let Some(component) = &mut component_modify {
                if let Some(lib) = &self.selected_variant_library {
                    let olib = c.libraries.get(lib);
                    if let Some(libr) = olib {
                        if let Some(library) = &libr.library {
                            let mut cb = egui::ComboBox::from_label("Select symbol from library");
                            if let Some(l) = &component.symbol {
                                cb = cb.selected_text(l.sym.clone());
                            }
                            cb.show_ui(ui, |ui| {
                                for l in library.syms.keys() {
                                    if ui.selectable_label(false, l).clicked() {
                                        let lr = if Some(lib.clone()) == self.selected_library {
                                            LibraryReference::ThisOne
                                        } else {
                                            LibraryReference::Another(lib.clone())
                                        };
                                        component.symbol = Some(crate::symbol::SymbolReference {
                                            lib: lr,
                                            sym: l.clone(),
                                        });
                                        component_changed = true;
                                    }
                                }
                            });
                        }
                    }
                }
            }
        });
        if component_changed {
            if let Some(component) = &mut component_modify {
                if let Some(lib) = &self.selected_library {
                    if let Some(Thing::Component(comm)) = &self.selected_thing {
                        if let Some(var) = &self.selected_variant {
                            actions.push(LibraryAction::ChangeComponentVariantSymbol {
                                libname: lib.clone(),
                                comname: comm.clone(),
                                varname: var.clone(),
                                sref: component.symbol.clone(),
                            });
                        }
                    }
                }
            }
        }
        for a in actions {
            c.library_log.apply(&mut c.libraries, a);
        }

        RedrawResponse {
            quit,
            new_windows: windows_to_create,
        }
    }
}
