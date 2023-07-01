//! This module contains definitions and code pertaining to schematic symbols and their definitions

use crate::library::LibraryAction;
use crate::schematic::TextOnPage;
use egui_multiwin::egui::{self, Color32};

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
    /// The origin modifier for panning the symbol around
    origin: &'a mut egui::Vec2,
    /// The zoom factor
    zoom: &'a mut f32,
    /// The component should be recentered
    recenter: bool,
}

impl<'a> SymbolDefinitionWidget<'a> {
    /// Create a widget that modifies a symbol definition
    pub fn new(
        sym: &'a mut SymbolDefinitionHolder<'a>,
        mm: &'a mut MouseMode,
        selection: &'a mut Vec<SymbolWidgetSelection>,
        actions: &'a mut Vec<LibraryAction>,
        origin: &'a mut egui::Vec2,
        zoom: &'a mut f32,
        recenter: bool,
    ) -> Self {
        Self {
            sym,
            mm,
            selection,
            actions,
            origin,
            zoom,
            recenter,
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
        let mut area = ui.cursor();
        area.max.x = ui.available_width() + area.min.x;
        area.max.y = ui.available_height() + area.min.y;
        let size = egui::vec2(area.max.x - area.min.x, area.max.y - area.min.y);
        if self.recenter {
            *self.origin = area.left_top().to_vec2() + egui::vec2(size.x / 2.0, size.y / 2.0);
        }

        let zoom_origin = area.left_top().to_vec2() + egui::vec2(size.x / 2.0, size.y / 2.0);

        let (mut pr, pntr) = ui.allocate_painter(size, sense);
        let color = egui::Color32::RED;

        match &self.mm {
            MouseMode::NewText | MouseMode::TextDrag => {
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    *self.mm = MouseMode::Selection;
                }
                self.selection.clear();
            }
            MouseMode::Selection => {
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.selection.clear();
                }
            }
        }

        let display_origin = (((*self.origin - zoom_origin) * *self.zoom) + zoom_origin).to_pos2();
        let stroke = egui_multiwin::egui::Stroke { width: 1.0, color: Color32::BLUE };
        pntr.line_segment(
            [
                egui::pos2(area.min.x, display_origin.y),
                egui::pos2(area.max.x, display_origin.y),
            ],
            stroke,
        );
        pntr.line_segment(
            [
                egui::pos2(display_origin.x, area.min.y),
                egui::pos2(display_origin.x, area.max.y),
            ],
            stroke,
        );

        if pr.clicked() {
            match self.mm {
                MouseMode::Selection => {
                    let inp = ui.input(|i| i.modifiers);
                    if !inp.shift && !inp.ctrl {
                        self.selection.clear();
                    }
                }
                _ => {}
            }
        }

        if let MouseMode::NewText = &self.mm {
            let pos = ui.input(|i| i.pointer.interact_pos());
            if let Some(pos) = pos {
                if pr.clicked() {
                    let pos2 = pos - area.left_top() - *self.origin;
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
                            size: 24.0 * *self.zoom,
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
                size: 24.0 * *self.zoom,
                family: egui::FontFamily::Monospace,
            };
            let temp = area.left_top() + pos + *self.origin;
            let temp = (((temp - zoom_origin).to_vec2() * *self.zoom) + zoom_origin).to_pos2();
            let color = t.color();
            let r = pntr.text(temp, align, t.text.clone(), font, color);
            let id = egui::Id::new(1 + i);
            let response = ui.interact(r, id, sense);
            let response = match self.mm {
                MouseMode::NewText => response,
                MouseMode::Selection => {
                    if response.clicked() {
                        let inp = ui.input(|i| i.modifiers);
                        if !inp.shift && !inp.ctrl {
                            self.selection.clear();
                        }
                        self.selection
                            .push(SymbolWidgetSelection::Text { textnum: i });
                    }
                    response.context_menu(|ui| {
                        if ui.button("Properties").clicked() {
                            ui.close_menu();
                        }
                    })
                }
                MouseMode::TextDrag => {
                    if response.dragged_by(egui::PointerButton::Primary) {
                        let amount = response.drag_delta() / *self.zoom;
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
                    })
                }
            };
            pr = pr.union(response);
        }

        let pr = pr.context_menu(|ui| {
            if ui.button("Do a thing").clicked() {
                ui.close_menu();
            }
            if ui.button("Close the menu").clicked() {
                ui.close_menu();
            }
        });

        let (_area, response) = ui.allocate_exact_size(size, sense);
        pr.union(response)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
/// The basic element of a schematic
pub struct Symbol {
    /// The list of free text that exists for the symbol
    pub texts: Vec<TextOnPage>,
}
