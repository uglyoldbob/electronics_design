//! This module contains definitions and code pertaining to schematic symbols and their definitions

use crate::library::LibraryAction;
use crate::schematic::TextOnPage;
use egui_multiwin::egui::{self, Color32};

#[derive(serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
/// Defines a pin for a symbol definition
pub struct Pin {
    /// The location of the pin
    pub location: crate::general::Coordinates,
    /// The rotation of the pin
    #[serde(default)]
    rotation: f32,
}

impl Pin {
    fn draw(&self, zoom: f32, pntr: &egui::Painter, pos: egui::Pos2) -> Vec<egui::Rect> {
        let mult = crate::general::Length::Inches(0.1).get_screen(zoom);
        let pos2 = pos
            + egui::Vec2 {
                x: self.rotation.to_radians().sin() * mult,
                y: self.rotation.to_radians().cos() * mult,
            };
        pntr.line_segment(
            [pos, pos2],
            egui::Stroke {
                width: 2.0,
                color: egui::Color32::WHITE,
            },
        );
        let rect = egui::Rect {
            min: (pos - crate::general::Coordinates::Inches(0.025, -0.025).get_pos2(zoom)).to_pos2(),
            max: (pos + crate::general::Coordinates::Inches(0.025, -0.025).get_pos2(zoom).to_vec2()),
        };
        println!("Line is {:?} {:?}", pos, pos2);
        println!("Rect box is {} {} {:?} {:?}", mult, zoom, pos, rect);
        pntr.rect_stroke(
            rect,
            0.0,
            egui::Stroke {
                width: 1.0,
                color: egui::Color32::WHITE,
            },
        );
        vec![rect]
    }

    fn respond(ui: &mut egui::Ui, id: String, rects: Vec<egui::Rect>) -> egui::Response {
        let mut resp = ui.interact(
            rects[0],
            egui::Id::from(format!("{}.{}", id, 0)),
            egui::Sense {
                click: true,
                drag: true,
                focusable: true,
            },
        );
        for (num, r) in rects.iter().skip(1).enumerate() {
            let num = num + 1;
            let resp2 = ui.interact(
                *r,
                egui::Id::from(format!("{}.{}", id, num)),
                egui::Sense {
                    click: true,
                    drag: true,
                    focusable: true,
                },
            );
            resp = resp.union(resp2);
        }
        resp
    }
}

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
    /// Allows creating new pins for a symbol, with a specified rotation
    NewPin,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
/// Defines what a symbol should look like
pub struct SymbolDefinition {
    /// The name of the Symbol
    name: String,
    /// The text in a symbol
    pub texts: Vec<TextOnPage>,
    /// The pins for a symbol
    #[serde(default)]
    pub pins: Vec<Pin>,
}

impl SymbolDefinition {
    /// Create a new named definition
    pub fn new(name: String) -> Self {
        Self {
            name,
            texts: Vec::new(),
            pins: Vec::new(),
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
    /// A pin of a symbol
    Pin {
        /// The pin identifier
        pinnum: usize,
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
    /// The angle to draw new pins at, in degrees
    pin_angle: &'a mut f32,
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
        pin_angle: &'a mut f32,
    ) -> Self {
        Self {
            sym,
            mm,
            selection,
            actions,
            origin,
            zoom,
            recenter,
            pin_angle,
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
            MouseMode::NewPin => {
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    *self.mm = MouseMode::Selection;
                } else if ui.input(|i| i.key_pressed(egui::Key::R)) {
                    let mut temp = *self.pin_angle + 90.0;
                    while temp > 360.0 {
                        temp -= 360.0;
                    }
                    *self.pin_angle = temp;
                }
            }
        }

        let stroke = egui_multiwin::egui::Stroke {
            width: 1.0,
            color: Color32::BLUE,
        };
        pntr.line_segment(
            [
                egui::pos2(area.min.x, self.origin.y),
                egui::pos2(area.max.x, self.origin.y),
            ],
            stroke,
        );
        pntr.line_segment(
            [
                egui::pos2(self.origin.x, area.min.y),
                egui::pos2(self.origin.x, area.max.y),
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

        for (i, t) in self.sym.sym.texts.iter().enumerate() {
            let pos = t.location.get_pos2(*self.zoom).to_vec2();
            let align = egui::Align2::LEFT_TOP;
            let font = egui::FontId {
                size: t.size.get_screen(*self.zoom),
                family: egui::FontFamily::Monospace,
            };
            let temp = pos.to_pos2() + *self.origin;
            let color = t.color();
            let r = pntr.text(temp, align, t.text.clone(), font, color);
            let id = egui::Id::new(1 + i);
            let response = ui.interact(r, id, sense);
            let response = match self.mm {
                MouseMode::NewPin => response,
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
                            delta: crate::general::Coordinates::from_pos2(
                                amount.to_pos2(),
                                *self.zoom,
                            ),
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

        for (i, p) in self.sym.sym.pins.iter().enumerate() {
            let pos = p.location.get_pos2(*self.zoom).to_vec2();
            let temp = pos + *self.origin;
            let rects = p.draw(*self.zoom, &pntr, temp.to_pos2());
            let response = crate::symbol::Pin::respond(ui, format!("pin {}", i), rects);
            let response = match self.mm {
                MouseMode::NewPin => response,
                MouseMode::NewText => response,
                MouseMode::Selection => {
                    if response.clicked() {
                        let inp = ui.input(|i| i.modifiers);
                        if !inp.shift && !inp.ctrl {
                            self.selection.clear();
                        }
                        self.selection
                            .push(SymbolWidgetSelection::Pin { pinnum: i });
                    }
                    response.context_menu(|ui| {
                        if ui.button("Properties").clicked() {
                            ui.close_menu();
                        }
                    })
                }
                MouseMode::TextDrag => response.context_menu(|ui| {
                    if ui.button("Properties").clicked() {
                        ui.close_menu();
                    }
                }),
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

        let pos = ui.input(|i| i.pointer.interact_pos());
        if let Some(pos) = pos {
            let pos2 = pos - *self.origin;
            match self.mm {
                MouseMode::Selection => {}
                MouseMode::TextDrag => {}
                MouseMode::NewText => {
                    if pr.clicked() {
                        self.actions.push(LibraryAction::CreateText {
                            libname: self.sym.libname.clone(),
                            symname: self.sym.sym.name.clone(),
                            text: TextOnPage {
                                text: "New text".to_string(),
                                location: crate::general::Coordinates::from_pos2(pos2, *self.zoom),
                                color: color.to_srgba_unmultiplied(),
                                size: crate::general::Length::Inches(0.2),
                            },
                        });
                    } else {
                        pntr.text(
                            pos,
                            egui::Align2::LEFT_TOP,
                            "New text".to_string(),
                            egui::FontId {
                                size: crate::general::Length::Inches(0.2).get_screen(*self.zoom),
                                family: egui::FontFamily::Monospace,
                            },
                            color,
                        );
                    }
                }
                MouseMode::NewPin => {
                    let pin = crate::symbol::Pin {
                        location: crate::general::Coordinates::from_pos2(pos2, *self.zoom),
                        rotation: *self.pin_angle,
                    };
                    if pr.clicked() {
                        self.actions.push(LibraryAction::CreatePin {
                            libname: self.sym.libname.clone(),
                            symname: self.sym.sym.name.clone(),
                            pin: Some(pin),
                        });
                    } else {
                        pin.draw(*self.zoom, &pntr, pos);
                    }
                }
            }
        }

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
