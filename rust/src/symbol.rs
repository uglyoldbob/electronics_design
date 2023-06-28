//! This module contains definitions and code pertaining to schematic symbols and their definitions

use crate::library::LibraryAction;
use crate::schematic::TextOnPage;
use egui_multiwin::egui;

/// Defines the mode for mouse interaction for symbols
#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
#[non_exhaustive]
pub enum MouseMode {
    /// Allows a user to select objects on the current schematic page
    Selection,
    /// Allows a user to drag text around on the current schematic page
    TextDrag,
    /// Allows new text to be placed on a page
    NewText,
}

#[derive(serde::Serialize, serde::Deserialize)]
/// Defines what a symbol should look like
pub struct SymbolDefinition {
    /// The name of the Symbol
    name: String,
    /// The text in a symbol
    pub texts: Vec<TextOnPage>,
}

impl SymbolDefinition {
    /// Create a new named definition
    pub fn new(name: String) -> Self {
        Self {
            name,
            texts: Vec::new(),
        }
    }
}

/// Separates stored and non-stored data for a symbol definition
pub struct SymbolDefinitionHolder<'a> {
    /// The symbol being held
    sym: &'a mut SymbolDefinition,
    /// The name of the containing library
    libname: String,
}

impl<'a> SymbolDefinitionHolder<'a> {
    /// Create a new symbol definition holder
    pub fn new(sym: &'a mut SymbolDefinition, libname: String) -> Self {
        Self { sym, libname }
    }
}

/// The possible objects to select in a symbol widget
pub enum SymbolWidgetSelection {
    /// A basic text field of a symbol
    Text {
        /// The text identifier
        textnum: usize,
    },
}

/// A Widget for modifying a symbol
pub struct SymbolDefinitionWidget<'a> {
    /// The symbol being modified by the widget
    sym: &'a mut SymbolDefinitionHolder<'a>,
    /// The mouse mode for the widget
    mm: &'a mut MouseMode,
    /// The currently selected symbol objects
    selection: &'a mut Vec<SymbolWidgetSelection>,
    /// The log for applying symbol modifications
    actions: &'a mut Vec<LibraryAction>,
}

impl<'a> SymbolDefinitionWidget<'a> {
    /// Create a widget that modifies a symbol definition
    pub fn new(
        sym: &'a mut SymbolDefinitionHolder<'a>,
        mm: &'a mut MouseMode,
        selection: &'a mut Vec<SymbolWidgetSelection>,
        actions: &'a mut Vec<LibraryAction>,
    ) -> Self {
        Self {
            sym,
            mm,
            selection,
            actions,
        }
    }
}

impl<'a> egui::Widget for SymbolDefinitionWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let sense = egui::Sense {
            click: true,
            drag: true,
            focusable: true,
        };
        let _context = ui.ctx();
        let mut area = ui.cursor();
        area.max.x = ui.available_width() + area.min.x;
        area.max.y = ui.available_height() + area.min.y;
        let size = egui::vec2(area.max.x - area.min.x, area.max.y - area.min.y);

        let (pr, pntr) = ui.allocate_painter(size, sense);
        let color = egui::Color32::RED;

        match &self.mm {
            MouseMode::NewText | MouseMode::TextDrag => {
                if ui.input().key_pressed(egui::Key::Escape) {
                    *self.mm = MouseMode::Selection;
                }
                self.selection.clear();
            }
            MouseMode::Selection => {
                if ui.input().key_pressed(egui::Key::Escape) {
                    self.selection.clear();
                }
            }
        }

        if pr.dragged_by(egui::PointerButton::Middle) {
            println!("what a drag");
        }

        if let MouseMode::NewText = &self.mm {
            let pos = ui.input().pointer.interact_pos();
            if let Some(pos) = pos {
                if pr.clicked() {
                    let pos2 = pos - area.left_top();
                    self.actions.push(LibraryAction::CreateText {
                        libname: self.sym.libname.clone(),
                        symname: self.sym.sym.name.clone(),
                        text: TextOnPage {
                            text: "New text".to_string(),
                            x: pos2.x,
                            y: pos2.y,
                            color: color.to_srgba_unmultiplied(),
                        },
                    });
                } else {
                    pntr.text(
                        pos,
                        egui::Align2::LEFT_TOP,
                        "New text".to_string(),
                        egui::FontId {
                            size: 24.0,
                            family: egui::FontFamily::Monospace,
                        },
                        color,
                    );
                }
            }
        }

        for (i, t) in self.sym.sym.texts.iter().enumerate() {
            let pos = egui::Vec2 { x: t.x, y: t.y };
            let align = egui::Align2::LEFT_TOP;
            let font = egui::FontId {
                size: 24.0,
                family: egui::FontFamily::Monospace,
            };
            let temp = area.left_top() + pos;
            let color = t.color();
            let r = pntr.text(temp, align, t.text.clone(), font, color);
            let id = egui::Id::new(1 + i);
            let response = ui.interact(r, id, sense);
            match self.mm {
                MouseMode::NewText => {}
                MouseMode::Selection => {
                    if response.clicked() {
                        println!("Clicked");
                        self.selection.clear();
                        self.selection
                            .push(SymbolWidgetSelection::Text { textnum: i });
                    }
                    response.context_menu(|ui| {
                        if ui.button("Properties").clicked() {
                            ui.close_menu();
                        }
                    });
                }
                MouseMode::TextDrag => {
                    if response.clicked() {
                        println!("Clicked");
                    }
                    if response.dragged() {
                        let amount = response.drag_delta();
                        let a = LibraryAction::MoveText {
                            libname: self.sym.libname.clone(),
                            symname: self.sym.sym.name.clone(),
                            textnum: i,
                            dx: amount.x,
                            dy: amount.y,
                        };
                        self.actions.push(a);
                    }
                    response.context_menu(|ui| {
                        if ui.button("Properties").clicked() {
                            ui.close_menu();
                        }
                    });
                }
            }
        }

        if pr.clicked() {
            match self.mm {
                MouseMode::Selection => {
                    self.selection.clear();
                }
                _ => {}
            }
        }

        pr.context_menu(|ui| {
            if ui.button("Do a thing").clicked() {
                ui.close_menu();
            }
            if ui.button("Close the menu").clicked() {
                ui.close_menu();
            }
        });

        let (_area, response) = ui.allocate_exact_size(
            size,
            egui::Sense {
                click: true,
                drag: true,
                focusable: true,
            },
        );
        response
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
/// The basic element of a schematic
pub struct Symbol {
    /// The list of free text that exists for the symbol
    pub texts: Vec<TextOnPage>,
}
