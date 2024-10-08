//! The schematic window is used to view and manipulate an electronic schematic.

use egui_multiwin::egui_glow::EguiGlow;
use egui_multiwin::egui;
use crate::egui_multiwin_dynamic::{
    multi_window::NewWindowRequest,
    tracked_window::{RedrawResponse, TrackedWindow},
};

use crate::component::ComponentVariantReference;
use crate::schematic::{MouseMode, Schematic, SchematicAction, SchematicHolder, SchematicWidget};
use crate::MyApp;

/// Defines messages that can some from other threads
enum Message {
    ///The schematic is being saved
    SaveSchematicName(crate::general::StoragePath),
    ///The schematic is being loaded
    LoadSchematicName(crate::general::StoragePath, crate::general::StorageFormat),
    /// Create a pdf of the current schematic
    CreatePdf(crate::general::StoragePath),
}

/// The window structure
pub struct SchematicWindow {
    /// Set when the title needs to change.
    new_title: Option<String>,
    /// The object currently selected, will eventually change to Vec<SchematicSelection>
    selection: Option<crate::schematic::SchematicSelection>,
    /// The message channel for communicating with the main thread, when needed.
    message_channel: (
        std::sync::mpsc::Sender<Message>,
        std::sync::mpsc::Receiver<Message>,
    ),
    /// The mouse mode for the schematic editor.
    mm: MouseMode,
    /// The origin for the symbol drawing
    origin: crate::general::Coordinates,
    /// The zoom factor for the widget
    zoom: f32,
    /// The name of the library selected
    selected_library: Option<String>,
    /// The component selected for adding to schematic
    selected_component: Option<String>,
    /// The variant of the component selected for addition to schematic
    selected_variant: Option<String>,
}

impl SchematicWindow {
    /// Create a new window
    pub fn request() -> NewWindowRequest {
        NewWindowRequest::new(
            super::Windows::Schematic(Self {
                new_title: None,
                selection: None,
                message_channel: std::sync::mpsc::channel(),
                mm: MouseMode::Selection,
                origin: crate::general::Coordinates::Inches(0.0, 0.0),
                zoom: 115.0,
                selected_library: None,
                selected_component: None,
                selected_variant: None,
            }),
            egui_multiwin::winit::window::WindowBuilder::new()
                .with_resizable(true)
                .with_inner_size(egui_multiwin::winit::dpi::LogicalSize {
                    width: 800.0,
                    height: 600.0,
                })
                .with_title("egui-multiwin root window"),
            egui_multiwin::tracked_window::TrackedWindowOptions {
                vsync: false,
                shader: None,
            },
            egui_multiwin::multi_window::new_id(),
        )
    }
}

impl TrackedWindow for SchematicWindow {
    fn is_root(&self) -> bool {
        true
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

        while let Ok(message) = self.message_channel.1.try_recv() {
            match message {
                Message::CreatePdf(path) => {
                    if let Some(sch) = &c.schematic {
                        let width =
                            printpdf::Mm(crate::general::Length::Inches(11.0).get_mm().into());
                        let height =
                            printpdf::Mm(crate::general::Length::Inches(11.0).get_mm().into());
                        let (doc, page1, layer1) =
                            printpdf::PdfDocument::new(sch.name(), width, height, "Layer 1");
                        let font = doc.add_external_font(crate::COMPUTER_MODERN_FONT).unwrap();
                        if !sch.schematic.pages.is_empty() {
                            let current_layer = doc.get_page(page1).get_layer(layer1);
                            sch.schematic.pages[0].draw_on(current_layer, &font);
                        }
                        for page in sch.schematic.pages[1..].iter() {
                            let (pdfpage, layer) = doc.add_page(width, height, "Layer 1");
                            let current_layer = doc.get_page(pdfpage).get_layer(layer);
                            page.draw_on(current_layer, &font);
                        }
                        let _e = doc.save(&mut std::io::BufWriter::new(path.writer().unwrap()));
                    }
                }
                Message::SaveSchematicName(n) => {
                    if let Some(s) = &mut c.schematic {
                        s.set_path(n);
                        if let Err(_e) = s.save() {
                            //TODO show the actual error to the user?
                            native_dialog::MessageDialog::new()
                                .set_type(native_dialog::MessageType::Error)
                                .set_title("ERROR")
                                .set_text("Unable to save file")
                                .show_alert()
                                .unwrap();
                        }
                    }
                }
                Message::LoadSchematicName(n, format) => match n.reader() {
                    Ok(mut reader) => match format.load::<Schematic>(&mut reader) {
                        Ok(sch) => {
                            c.schematic = Some(SchematicHolder {
                                schematic: sch,
                                schematic_log: undo::Record::new(),
                                schematic_was_saved: false,
                                path: Some(n),
                                format,
                            });
                        }
                        Err(e) => {
                            let _ = native_dialog::MessageDialog::new()
                                .set_title("Failed to open schematic")
                                .set_text(&e.to_string())
                                .show_alert();
                        }
                    },
                    Err(e) => {
                        let _ = native_dialog::MessageDialog::new()
                            .set_title("Failed to open schematic")
                            .set_text(&e.to_string())
                            .show_alert();
                    }
                },
            }
        }

        if let Some(s) = &mut c.schematic {
            s.check_for_saved_status_change(|s, b| {
                println!("New save status is {}", b);
                let mut s: String = s.name();
                if !b {
                    s.push('*');
                }
                self.new_title = Some(s);
            });
        } else {
            self.new_title = Some(crate::PACKAGE_NAME.to_string());
        }

        egui::TopBottomPanel::top("menubar").show(&egui.egui_ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .add_enabled(c.schematic.is_none(), egui::Button::new("New schematic"))
                        .clicked()
                    {
                        c.schematic = Some(SchematicHolder::new_example());
                        ui.close_menu();
                    }
                    if ui
                        .add_enabled(c.schematic.is_none(), egui::Button::new("Load schematic"))
                        .clicked()
                    {
                        let f = rfd::AsyncFileDialog::new()
                            .add_filter("Raw", &["urf"])
                            .set_title("Load schematic")
                            .pick_file();
                        let message_sender = self.message_channel.0.clone();
                        crate::execute(async move {
                            let file = f.await;
                            if let Some(file) = file {
                                let mut fname = file.path().to_path_buf();
                                fname.set_extension("urf");
                                message_sender
                                    .send(Message::LoadSchematicName(
                                        crate::general::StoragePath::LocalFilesystem(
                                            fname.into_os_string().into_string().unwrap(),
                                        ),
                                        crate::general::StorageFormat::default(),
                                    ))
                                    .ok();
                            }
                        });
                        ui.close_menu();
                    }
                    if ui
                        .add_enabled(c.schematic.is_some(), egui::Button::new("Save schematic"))
                        .clicked()
                    {
                        if let Some(s) = &mut c.schematic {
                            if s.has_path() {
                                if let Err(e) = s.save() {
                                    let s: String = format!("Unable to save file {}", e);
                                    native_dialog::MessageDialog::new()
                                        .set_type(native_dialog::MessageType::Error)
                                        .set_title("ERROR")
                                        .set_text(&s[..])
                                        .show_alert()
                                        .unwrap();
                                    unimplemented!();
                                }
                            } else {
                                let f = rfd::AsyncFileDialog::new()
                                    .add_filter("Raw", &["urf"])
                                    .set_title("Save schematic")
                                    .save_file();
                                let message_sender = self.message_channel.0.clone();
                                crate::execute(async move {
                                    let file = f.await;
                                    if let Some(file) = file {
                                        let mut fname = file.path().to_path_buf();
                                        fname.set_extension("urf");
                                        message_sender
                                            .send(Message::SaveSchematicName(
                                                crate::general::StoragePath::LocalFilesystem(
                                                    fname.into_os_string().into_string().unwrap(),
                                                ),
                                            ))
                                            .ok();
                                    }
                                });
                            }
                        }
                        ui.close_menu();
                    }
                    if ui
                        .add_enabled(c.schematic.is_some(), egui::Button::new("Close schematic"))
                        .clicked()
                    {
                        if let Some(s) = &c.schematic {
                            if s.has_unsaved_changes() {
                                unimplemented!();
                            } else {
                                c.schematic = None;
                            }
                        }
                        ui.close_menu();
                    }
                    if ui
                        .add_enabled(c.schematic.is_some(), egui::Button::new("Export to pdf"))
                        .clicked()
                    {
                        let f = rfd::AsyncFileDialog::new()
                            .add_filter("Pdf", &["pdf"])
                            .set_title("Export schematic as pdf")
                            .save_file();
                        let message_sender = self.message_channel.0.clone();
                        crate::execute(async move {
                            let file = f.await;
                            if let Some(file) = file {
                                let mut fname = file.path().to_path_buf();
                                fname.set_extension("pdf");
                                message_sender
                                    .send(Message::CreatePdf(
                                        crate::general::StoragePath::LocalFilesystem(
                                            fname.into_os_string().into_string().unwrap(),
                                        ),
                                    ))
                                    .ok();
                            }
                        });
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
                let (undoable, redoable) = if let Some(sch) = &mut c.schematic {
                    (sch.schematic_log.can_undo(), sch.schematic_log.can_redo())
                } else {
                    (false, false)
                };
                ui.menu_button("Edit", |ui| {
                    if ui
                        .add_enabled(undoable, egui::Button::new("Undo"))
                        .clicked()
                    {
                        if let Some(sch) = &mut c.schematic {
                            sch.schematic_log.undo(&mut sch.schematic);
                        }
                        ui.close_menu();
                    }
                    if ui
                        .add_enabled(redoable, egui::Button::new("Redo"))
                        .clicked()
                    {
                        if let Some(sch) = &mut c.schematic {
                            sch.schematic_log.redo(&mut sch.schematic);
                        }
                        ui.close_menu();
                    }
                });
            });
        });

        egui::TopBottomPanel::top("button bar").show(&egui.egui_ctx, |ui| {
            if c.schematic.is_some() {
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
                    ui.selectable_value(&mut self.mm, MouseMode::NewComponent, "C")
                        .on_hover_ui(|ui| {
                            ui.label("Add component mode");
                        });
                });
            }
        });

        egui::SidePanel::left("left panel")
            .resizable(true)
            .show(&egui.egui_ctx, |ui| {
                egui::TopBottomPanel::top("library select")
                    .resizable(true)
                    .show_inside(ui, |ui| {
                        ui.label("Libraries");
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .scroll_bar_visibility(
                                egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                            )
                            .auto_shrink([false, false])
                            .stick_to_right(true)
                            .show(ui, |ui| {
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

                if let Some(l) = &self.selected_library {
                    let check = c.libraries.get_mut(l);
                    if let Some(lib) = check {
                        if let Some(library) = &lib.library {
                            egui::TopBottomPanel::top("component select")
                                .resizable(true)
                                .show_inside(ui, |ui| {
                                    ui.label("Components");
                                    ui.separator();
                                    egui::ScrollArea::vertical()
                                        .id_source("component scroll")
                                        .scroll_bar_visibility(
                                            egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                                        )
                                        .auto_shrink([false, false])
                                        .stick_to_right(true)
                                        .show(ui, |ui| {
                                            for name in library.components.keys() {
                                                if ui
                                                    .selectable_label(
                                                        self.selected_component == Some(name.clone()),
                                                        name,
                                                    )
                                                    .clicked()
                                                {
                                                    self.selected_component = Some(name.clone());
                                                }
                                            }
                                        });
                                });
                            if let Some(component) = &self.selected_component {
                                if let Some(component) = library.components.get(component) {
                                    egui::TopBottomPanel::top("variant select")
                                        .resizable(true)
                                        .show_inside(ui, |ui| {
                                            ui.label("Variants");
                                            ui.separator();
                                            egui::ScrollArea::vertical()
                                                .id_source("variant scroll")
                                                .scroll_bar_visibility(
                                                    egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                                                )
                                                .auto_shrink([false, false])
                                                .stick_to_right(true)
                                                .show(ui, |ui| {
                                                    for name in component.variants.keys() {
                                                        if ui
                                                            .selectable_label(
                                                                self.selected_variant == Some(name.clone()),
                                                                name,
                                                            )
                                                            .clicked()
                                                        {
                                                            self.selected_variant = Some(name.clone());
                                                            self.mm = MouseMode::NewComponent;
                                                        }
                                                    }
                                                });
                                        });
                                }
                            }
                        }
                    }
                }
            });

        let mut actionlog = Vec::new();

        egui::SidePanel::right("right panel")
            .resizable(true)
            .show(&egui.egui_ctx, |ui| {
                ui.label("Right");
                if let Some(sch) = &mut c.schematic {
                    if let Some(sel) = &self.selection {
                        match sel {
                            crate::schematic::SchematicSelection::Symbol { page, sym } => {
                                let var_ref = &sch.schematic.pages[*page].syms[*sym];
                                let units = var_ref.pos.get_units(c.units);
                                let mut xstr = format!("{:.4}", units.0);
                                ui.horizontal(|ui| {
                                    ui.label("X ");
                                    ui.add(egui::TextEdit::singleline(&mut xstr));
                                });
                                if let Ok(x) = xstr.parse::<f32>() {
                                    actionlog.push(SchematicAction::MoveSymbol {
                                        pagenum: *page,
                                        symnum: *sym,
                                        delta: crate::general::Coordinates::from_pos2(
                                            egui::pos2(x - units.0, 0.0),
                                            1.0,
                                        ),
                                    });
                                }
                                let mut ystr = format!("{:.4}", units.1);
                                ui.horizontal(|ui| {
                                    ui.label("Y ");
                                    ui.add(egui::TextEdit::singleline(&mut ystr));
                                });
                                if let Ok(y) = ystr.parse::<f32>() {
                                    actionlog.push(SchematicAction::MoveSymbol {
                                        pagenum: *page,
                                        symnum: *sym,
                                        delta: crate::general::Coordinates::from_pos2(
                                            egui::pos2(0.0, units.1 - y),
                                            1.0,
                                        ),
                                    });
                                }
                                ui.label("A symbol has been selected");
                            }
                            crate::schematic::SchematicSelection::Text { page, textnum } => {
                                let t = &sch.schematic.pages[*page].texts[*textnum];
                                ui.label("Text Properties");
                                let mut text = t.text.clone();
                                ui.horizontal(|ui| {
                                    ui.label("Text ");
                                    ui.add(egui::TextEdit::singleline(&mut text));
                                });
                                if text != t.text {
                                    actionlog.push(SchematicAction::EditText {
                                        pagenum: *page,
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
                                    actionlog.push(SchematicAction::MoveText {
                                        pagenum: *page,
                                        textnum: *textnum,
                                        delta: crate::general::Coordinates::from_pos2(
                                            egui::pos2(x - units.0, 0.0),
                                            1.0,
                                        ),
                                    });
                                }
                                let mut ystr = format!("{:.4}", units.1);
                                ui.horizontal(|ui| {
                                    ui.label("Y ");
                                    ui.add(egui::TextEdit::singleline(&mut ystr));
                                });
                                if let Ok(y) = ystr.parse::<f32>() {
                                    actionlog.push(SchematicAction::MoveText {
                                        pagenum: *page,
                                        textnum: *textnum,
                                        delta: crate::general::Coordinates::from_pos2(
                                            egui::pos2(0.0, units.1 - y),
                                            1.0,
                                        ),
                                    });
                                }
                                let mut color = t
                                    .color
                                    .get_color32(crate::general::ColorMode::ScreenModeDark);
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    actionlog.push(SchematicAction::ChangeTextColor {
                                        pagenum: *page,
                                        textnum: *textnum,
                                        old: t.color,
                                        new: crate::schematic::Colors::Custom(
                                            color.to_srgba_unmultiplied(),
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }
            });

        if let Some(sch) = &mut c.schematic {
            for a in actionlog {
                sch.schematic_log.apply(&mut sch.schematic, a);
            }
        }

        let mut component: Option<crate::component::ComponentVariantReference> = None;
        if let Some(libname) = &self.selected_library {
            if let Some(sch) = &self.selected_component {
                if let Some(var) = &self.selected_variant {
                    component = Some(ComponentVariantReference {
                        lib: libname.to_owned(),
                        com: sch.to_owned(),
                        var: var.to_owned(),
                        pos: crate::general::Coordinates::Inches(0.0, 0.0),
                    });
                }
            }
        }

        egui::CentralPanel::default().show(&egui.egui_ctx, |ui| {
            if let Some(sch) = &mut c.schematic {
                let sch = SchematicWidget::new(
                    sch,
                    &mut self.mm,
                    &mut self.selection,
                    &mut self.origin,
                    &mut self.zoom,
                    component,
                    &c.libraries,
                );
                let resp = ui.add(sch);
                if resp.dragged_by(egui::PointerButton::Middle) {
                    self.origin += crate::general::Coordinates::from_pos2(
                        resp.drag_delta().to_pos2(),
                        self.zoom,
                    );
                }
                if resp.double_clicked_by(egui::PointerButton::Middle) {
                    //self.recenter = true;
                }
                if resp.hovered() {
                    let scroll = ui.input(|i| i.smooth_scroll_delta);
                    if scroll.y.abs() > f32::EPSILON {
                        self.zoom *= f32::powf(1.0025, scroll.y);
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
