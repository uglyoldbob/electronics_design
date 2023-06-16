#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod schematic;
use crate::schematic::Schematic;
use crate::schematic::SchematicWidget;

use eframe::egui;
use schematic::MouseMode;

fn main() {
    let options = eframe::NativeOptions::default();
    if let Err(e) = eframe::run_native(
        "UglyOldBob Electronics",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    ) {
        println!("Failed to run application: {}", e);
    }
}

enum Message {
    SchematicName(String),
}

struct ModalMessage {
    message: String,
    title: String,
}

impl eframe::App for ModalMessage {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

    }
}

struct MyApp {
    modal_window: Option<ModalMessage>,
    schematic: Option<Schematic>,
    path: Option<String>,
    mm: MouseMode,
    message_channel: (
        std::sync::mpsc::Sender<Message>,
        std::sync::mpsc::Receiver<Message>,
    ),
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: std::future::Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || futures::executor::block_on(f));
}
#[cfg(target_arch = "wasm32")]
fn execute<F: std::future::Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

impl MyApp {
    fn run_menu(&mut self, ui: &mut eframe::egui::Ui) {
        ui.menu_button("File", |ui| {
            if ui
                .add_enabled(self.schematic.is_none(), egui::Button::new("New schematic"))
                .clicked()
            {
                self.schematic = Some(Schematic::new());
                ui.close_menu();
            }
            if ui
                .add_enabled(
                    self.schematic.is_some(),
                    egui::Button::new("Save schematic"),
                )
                .clicked()
            {
                if let Some(s) = &mut self.schematic {
                    if let Some(p) = &self.path {
                        if let Err(e) = s.save(p) {
                            println!("Error saving file {:?}", e);
                            unimplemented!();
                        }
                    } else {
                        let f = rfd::AsyncFileDialog::new()
                            .add_filter("Raw", &["urf"])
                            .set_title("Save schematic")
                            .save_file();
                        let message_sender = self.message_channel.0.clone();
                        execute(async move {
                            let file = f.await;
                            if let Some(file) = file {
                                println!("Save into {}", file.file_name());
                                let mut fname = file.path().to_path_buf();
                                fname.set_extension("urf");
                                message_sender
                                    .send(Message::SchematicName(
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
                .add_enabled(
                    self.schematic.is_some(),
                    egui::Button::new("Close schematic"),
                )
                .clicked()
            {
                //TODO check for unsaved changes
                self.schematic = None;
                self.path = None;
                ui.close_menu();
            }
            ui.menu_button("Recent", |ui| {
                if ui.button("Thing 1").clicked() {
                    ui.close_menu();
                }
            });
        });
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            schematic: None,
            path: None,
            mm: MouseMode::Selection,
            message_channel: std::sync::mpsc::channel(),
            modal_window: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(modal) = &mut self.modal_window {
            egui::Window::new("Modal Window")
                .show(ctx, |ui| {
                    ui.label(&modal.title);
                    ui.label(&modal.message);
                });
        }
        while let Ok(message) = self.message_channel.1.try_recv() {
            match message {
                Message::SchematicName(n) => {
                    if let Some(s) = &mut self.schematic {
                        self.path = Some(n);
                        if let Some(p) = &self.path {
                            if let Err(e) = s.save(p) {
                                println!("Error saving file {:?}", e);
                                self.modal_window = Some(ModalMessage {
                                    title: "Test".to_string(),
                                    message: "Failed to save file".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        egui::TopBottomPanel::top("menubar").show(ctx, |ui| {
            self.run_menu(ui);
        });

        egui::TopBottomPanel::top("button bar").show(ctx, |ui| {
            if let Some(m) = &mut self.schematic {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.mm, MouseMode::Selection, "S");
                    ui.selectable_value(&mut self.mm, MouseMode::TextDrag, "T");
                    if ui.button("X").clicked() {
                        self.modal_window = Some(ModalMessage {
                            title: "Test".to_string(),
                            message: "Failed to save file".to_string(),
                        });
                    }
                });
            }
        });

        egui::SidePanel::left("left panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("Left");
            });

        egui::SidePanel::right("right panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("Right");
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(sch) = &mut self.schematic {
                let sch = SchematicWidget::new(sch, &mut self.mm);
                ui.add(sch);
            }
        });
    }
}
