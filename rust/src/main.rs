#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod schematic;
use crate::schematic::Schematic;

use eframe::egui;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

struct MyApp {
    name: String,
    age: u32,
}

impl MyApp {
    fn run_menu(&mut self, ui: &mut eframe::egui::Ui) {
        ui.menu_button("File", |ui| {
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
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.run_menu(ui);
            ui.heading("Electronics Design");
            ui.horizontal(|ui| {
                ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
            let sch = Schematic::new();
            ui.label("ABOVE");
            ui.horizontal_top(|ui| {
                ui.label("LEFT");
                ui.add(sch);
                ui.label("RIGHT");
            });
            ui.label("BELOW");
        });
    }
}
