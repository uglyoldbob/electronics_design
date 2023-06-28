//! The schematic window is used to view and manipulate an electronic schematic.

use std::io::Read;

use egui_multiwin::egui_glow::EguiGlow;
use egui_multiwin::{
    egui,
    multi_window::NewWindowRequest,
    tracked_window::{RedrawResponse, TrackedWindow},
};

use crate::schematic::{MouseMode, SchematicAction, SchematicHolder, SchematicWidget};
use crate::MyApp;

/// Defines messages that can some from other threads
enum Message {
    ///The schematic is being saved to disk with the specified filename
    SaveSchematicName(String),
    ///The schematic is being loaded from disk with the specified filename
    LoadSchematicName(String),
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
}

impl SchematicWindow {
    /// Create a new window
    pub fn request() -> NewWindowRequest<MyApp> {
        NewWindowRequest {
            window_state: Box::new(Self {
                new_title: None,
                selection: None,
                message_channel: std::sync::mpsc::channel(),
                mm: MouseMode::Selection,
            }),
            builder: egui_multiwin::glutin::window::WindowBuilder::new()
                .with_resizable(true)
                .with_inner_size(egui_multiwin::glutin::dpi::LogicalSize {
                    width: 800.0,
                    height: 600.0,
                })
                .with_title("egui-multiwin root window"),
            options: egui_multiwin::tracked_window::TrackedWindowOptions {
                vsync: false,
                shader: None,
            },
        }
    }
}

impl TrackedWindow for SchematicWindow {
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

        let windows_to_create = vec![];

        while let Ok(message) = self.message_channel.1.try_recv() {
            match message {
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
                Message::LoadSchematicName(n) => {
                    let f = std::fs::File::open(&n);
                    if let Ok(mut f) = f {
                        let meta = f.metadata();
                        if let Ok(meta) = meta {
                            let mut buffer = vec![0; meta.len() as usize];
                            let result = f.read(&mut buffer);
                            if let Ok(_result) = result {
                                c.schematic = SchematicHolder::load(n, &buffer);
                            }
                        }
                    }
                }
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
                                        fname.into_os_string().into_string().unwrap(),
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
                                    let s: String = format!("Unable to save file {:?}", e);
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
                                                fname.into_os_string().into_string().unwrap(),
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
                });
            }
        });

        egui::SidePanel::left("left panel")
            .resizable(true)
            .show(&egui.egui_ctx, |ui| {
                ui.label("Left");
            });

        let mut actionlog = Vec::new();

        egui::SidePanel::right("right panel")
            .resizable(true)
            .show(&egui.egui_ctx, |ui| {
                ui.label("Right");
                if let Some(sch) = &mut c.schematic {
                    if let Some(sel) = &self.selection {
                        match sel {
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
                                let mut xstr = format!("{:.4}", t.x);
                                ui.horizontal(|ui| {
                                    ui.label("X ");
                                    ui.add(egui::TextEdit::singleline(&mut xstr));
                                });
                                if let Ok(x) = xstr.parse::<f32>() {
                                    actionlog.push(SchematicAction::MoveText {
                                        pagenum: *page,
                                        textnum: *textnum,
                                        dx: x - t.x,
                                        dy: 0.0,
                                    });
                                }
                                let mut ystr = format!("{:.4}", t.y);
                                ui.horizontal(|ui| {
                                    ui.label("Y ");
                                    ui.add(egui::TextEdit::singleline(&mut ystr));
                                });
                                if let Ok(y) = ystr.parse::<f32>() {
                                    actionlog.push(SchematicAction::MoveText {
                                        pagenum: *page,
                                        textnum: *textnum,
                                        dx: 0.0,
                                        dy: y - t.y,
                                    });
                                }
                                let mut color = t.color();
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    actionlog.push(SchematicAction::ChangeTextColor {
                                        pagenum: *page,
                                        textnum: *textnum,
                                        old: t.color(),
                                        new: color,
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

        egui::CentralPanel::default().show(&egui.egui_ctx, |ui| {
            if let Some(sch) = &mut c.schematic {
                let sch = SchematicWidget::new(sch, &mut self.mm, &mut self.selection);
                ui.add(sch);
            }
        });

        RedrawResponse {
            quit,
            new_windows: windows_to_create,
        }
    }
}