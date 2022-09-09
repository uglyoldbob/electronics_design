#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod schematic;
use crate::schematic::Schematic;
use crate::schematic::SchematicWidget;

use eframe::egui;
use schematic::MouseMode;

fn main() {
    let mut options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

struct MyApp {
    name: String,
    age: u32,
    schematic: Schematic,
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
            schematic: Schematic::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menubar").show(ctx, |ui| {
            self.run_menu(ui);
        });

        egui::TopBottomPanel::top("button bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.schematic.mm, MouseMode::Selection, "S");
                ui.selectable_value(&mut self.schematic.mm, MouseMode::TextDrag, "T");
            });
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
            let sch = SchematicWidget::new(&mut self.schematic);
            ui.add(sch);
        });
    }
}
