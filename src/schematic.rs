//! The schematic module covers code related to a electronics schematic, consisting of one or more pages of stuff.

use std::collections::HashMap;

use egui_multiwin::egui::{self, Rect};

use crate::{
    component::ComponentVariantReference, general::StoragePath, library::LibraryHolder,
    symbol::Symbol,
};

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq)]
#[serde(tag = "type", content = "args")]
#[non_exhaustive]
/// The kinds of colors to use on a schematic
pub enum Colors {
    /// The standard color for visibility
    Standard,
    /// The color for the paper border
    Border,
    /// A custom color. See [egui::Color32] for the `to_srgba_unmultiplied` function
    Custom([u8; 4]),
}

impl Colors {
    /// Returns the specific color for the color specified and the given color mode
    pub fn get_color32(&self, mode: crate::general::ColorMode) -> egui::Color32 {
        match self {
            Colors::Standard => match mode {
                crate::general::ColorMode::ScreenModeDark => egui::Color32::from_rgb(255, 255, 255),
                crate::general::ColorMode::ScreenModeLight => egui::Color32::from_rgb(0, 0, 0),
                crate::general::ColorMode::PrintingMode => egui::Color32::from_rgb(0, 0, 0),
            },
            Colors::Border => match mode {
                crate::general::ColorMode::ScreenModeDark => egui::Color32::from_rgb(0, 0, 255),
                crate::general::ColorMode::ScreenModeLight => egui::Color32::from_rgb(0, 0, 255),
                crate::general::ColorMode::PrintingMode => egui::Color32::from_rgb(255, 255, 255),
            },
            Colors::Custom(c) => egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[non_exhaustive]
/// Represents free text anywhere on a page or symbol
pub struct TextOnPage {
    /// The text to display
    pub text: String,
    /// The location of the text
    pub location: crate::general::Coordinates,
    /// The color for the text.
    pub color: Colors,
    /// The size of the text in physical dimensions
    pub size: crate::general::Length,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
/// A single page of an electronic schematic
pub struct Page {
    /// The symbols on the page
    pub syms: Vec<ComponentVariantReference>,
    /// The free text items on the page
    pub texts: Vec<TextOnPage>,
    /// The physical size of the page
    pub size: crate::general::Coordinates,
}

impl Page {
    /// Draw the page on the given pdf layer
    pub fn draw_on(&self, layer: printpdf::PdfLayerReference, font: &printpdf::IndirectFontRef) {
        let points1 = vec![
            (
                printpdf::Point::new(printpdf::Mm(10.0), printpdf::Mm(10.0)),
                false,
            ),
            (
                printpdf::Point::new(printpdf::Mm(10.0), printpdf::Mm(20.0)),
                false,
            ),
            (
                printpdf::Point::new(printpdf::Mm(30.0), printpdf::Mm(20.0)),
                false,
            ),
            (
                printpdf::Point::new(printpdf::Mm(30.0), printpdf::Mm(10.0)),
                false,
            ),
        ];
        let line = printpdf::Line {
            points: points1,
            is_closed: true,
            has_fill: false,
            has_stroke: true,
            is_clipping_path: false,
        };
        layer.add_shape(line);
        for text in &self.texts {
            let (x, y) = text.location.get_mm();
            layer.use_text(
                text.text.clone(),
                (text.size.get_mm() * 2.85).into(),
                printpdf::Mm(x.into()),
                printpdf::Mm(y.into()),
                font,
            );
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
/// Represents an entire electronic schematic
pub struct Schematic {
    /// The list of pages for the schematic
    pub pages: Vec<Page>,
    /// The name of the schematic
    name: String,
}

/// Defines the mode for mouse interaction for schematics
#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
#[non_exhaustive]
pub enum MouseMode {
    /// Allows a user to select objects on the current schematic page
    Selection,
    /// Allows a user to drag text around on the current schematic page
    TextDrag,
    /// Allows new text to be placed on a page
    NewText,
    /// Allows a user to add components to a schematic
    NewComponent,
}

impl Schematic {
    /// Create a new example schematic.
    pub fn new_example() -> Self {
        let mut p = Vec::new();
        let t = vec![
            TextOnPage {
                text: "demo text".to_string(),
                location: crate::general::Coordinates::Inches(0.0, 0.0),
                color: Colors::Standard,
                size: crate::general::Length::Inches(0.2),
            },
            TextOnPage {
                text: "moredemo text".to_string(),
                location: crate::general::Coordinates::Inches(0.0, 0.2),
                color: Colors::Standard,
                size: crate::general::Length::Inches(0.4),
            },
            TextOnPage {
                text: "moredemo text".to_string(),
                location: crate::general::Coordinates::Inches(0.0, 0.6),
                color: Colors::Standard,
                size: crate::general::Length::Inches(0.8),
            },
            TextOnPage {
                text: "moredemo text".to_string(),
                location: crate::general::Coordinates::Inches(0.0, 1.4),
                color: Colors::Standard,
                size: crate::general::Length::Inches(1.6),
            },
            TextOnPage {
                text: "moredemo text".to_string(),
                location: crate::general::Coordinates::Inches(0.0, 3.0),
                color: Colors::Standard,
                size: crate::general::Length::Inches(3.2),
            },
        ];
        let page = Page {
            syms: Vec::new(),
            texts: t,
            size: crate::general::Coordinates::Inches(11.0, 8.5),
        };
        p.push(page);
        Self {
            pages: p,
            name: "Example Schematic".to_string(),
        }
    }
}

#[derive(Clone)]
/// The actions that can be done to a schematic. This allows the undo/redo functionality to exist.
pub enum SchematicAction {
    /// Move a symbol on the page by a certain amount
    MoveSymbol {
        /// The page number
        pagenum: usize,
        /// The synbol number
        symnum: usize,
        /// The delta to move by
        delta: crate::general::Coordinates,
    },
    /// Move text on a schematic page by a certain amount
    MoveText {
        /// The page number
        pagenum: usize,
        /// The text number
        textnum: usize,
        /// The delta to move by
        delta: crate::general::Coordinates,
    },
    /// Create a new text for a schematic
    CreateText {
        /// The page number
        pagenum: usize,
        /// The new text
        text: TextOnPage,
    },
    /// Edits the text of a text object for a schematic page
    EditText {
        /// The page number
        pagenum: usize,
        /// The text number
        textnum: usize,
        /// The old text
        old: String,
        /// The new text
        new: String,
    },
    /// Change the text color of a text object for a schematic page
    ChangeTextColor {
        /// The page number
        pagenum: usize,
        /// The text number
        textnum: usize,
        /// The old text
        old: Colors,
        /// The new text
        new: Colors,
    },
    /// Add a component variant to the schematic
    AddComponentVariant {
        /// The page number
        pagenum: usize,
        /// The variant
        var: ComponentVariantReference,
    },
}

impl undo::Action for SchematicAction {
    type Target = Schematic;

    type Output = ();

    fn apply(&mut self, target: &mut Self::Target) -> Self::Output {
        match self {
            SchematicAction::AddComponentVariant { pagenum, var } => {
                target.pages[*pagenum].syms.push(var.to_owned());
            }
            SchematicAction::MoveSymbol {
                pagenum,
                symnum,
                delta,
            } => {
                target.pages[*pagenum].syms[*symnum].pos += *delta;
            }
            SchematicAction::MoveText {
                pagenum,
                textnum,
                delta,
            } => {
                target.pages[*pagenum].texts[*textnum].location += *delta;
            }
            SchematicAction::CreateText { pagenum, text } => {
                target.pages[*pagenum].texts.push(text.clone());
            }
            SchematicAction::EditText {
                pagenum,
                textnum,
                old: _,
                new,
            } => {
                target.pages[*pagenum].texts[*textnum].text = new.clone();
            }
            SchematicAction::ChangeTextColor {
                pagenum,
                textnum,
                old: _,
                new,
            } => {
                target.pages[*pagenum].texts[*textnum].color = *new;
            }
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        match self {
            SchematicAction::AddComponentVariant { pagenum, var } => {
                *var = target.pages[*pagenum].syms.pop().unwrap();
            }
            SchematicAction::MoveSymbol {
                pagenum,
                symnum,
                delta,
            } => {
                target.pages[*pagenum].syms[*symnum].pos -= *delta;
            }
            SchematicAction::MoveText {
                pagenum,
                textnum,
                delta,
            } => {
                target.pages[*pagenum].texts[*textnum].location -= *delta;
            }
            SchematicAction::CreateText { pagenum, text: _ } => {
                target.pages[*pagenum].texts.pop();
            }
            SchematicAction::EditText {
                pagenum,
                textnum,
                old,
                new: _,
            } => {
                target.pages[*pagenum].texts[*textnum].text = old.clone();
            }
            SchematicAction::ChangeTextColor {
                pagenum,
                textnum,
                old,
                new: _,
            } => {
                target.pages[*pagenum].texts[*textnum].color = *old;
            }
        }
    }

    fn merge(&mut self, other: Self) -> undo::Merged<Self>
    where
        Self: Sized,
    {
        match self {
            SchematicAction::AddComponentVariant { pagenum: _, var: _ } => undo::Merged::No(other),
            SchematicAction::MoveSymbol {
                pagenum,
                symnum,
                delta,
            } => {
                if let SchematicAction::MoveSymbol {
                    pagenum: pn2,
                    symnum: sn2,
                    delta: delta2,
                } = other.clone()
                {
                    if *pagenum == pn2 && *symnum == sn2 {
                        if (*delta + delta2).less_than_epsilon() {
                            undo::Merged::Annul
                        } else {
                            *delta += delta2;
                            undo::Merged::Yes
                        }
                    } else {
                        undo::Merged::No(other)
                    }
                } else {
                    undo::Merged::No(other)
                }
            }
            SchematicAction::MoveText {
                pagenum,
                textnum,
                delta,
            } => {
                if let SchematicAction::MoveText {
                    pagenum: pn2,
                    textnum: tn2,
                    delta: delta2,
                } = other.clone()
                {
                    if *pagenum == pn2 && *textnum == tn2 {
                        if (*delta + delta2).less_than_epsilon() {
                            undo::Merged::Annul
                        } else {
                            *delta += delta2;
                            undo::Merged::Yes
                        }
                    } else {
                        undo::Merged::No(other)
                    }
                } else {
                    undo::Merged::No(other)
                }
            }
            SchematicAction::CreateText {
                pagenum: _,
                text: _,
            } => undo::Merged::No(other),
            SchematicAction::EditText {
                pagenum,
                textnum,
                old,
                new,
            } => {
                if let SchematicAction::EditText {
                    pagenum: pagenum2,
                    textnum: textnum2,
                    old: _,
                    new: new2,
                } = other.clone()
                {
                    if *pagenum == pagenum2 && *textnum == textnum2 {
                        if *old == new2 {
                            undo::Merged::Annul
                        } else {
                            *new = new2;
                            undo::Merged::Yes
                        }
                    } else {
                        undo::Merged::No(other)
                    }
                } else {
                    undo::Merged::No(other)
                }
            }
            SchematicAction::ChangeTextColor {
                pagenum,
                textnum,
                old,
                new,
            } => {
                if let SchematicAction::ChangeTextColor {
                    pagenum: pagenum2,
                    textnum: textnum2,
                    old: _,
                    new: new2,
                } = other.clone()
                {
                    if *pagenum == pagenum2 && *textnum == textnum2 {
                        if *old == new2 {
                            undo::Merged::Annul
                        } else {
                            *new = new2;
                            undo::Merged::Yes
                        }
                    } else {
                        undo::Merged::No(other)
                    }
                } else {
                    undo::Merged::No(other)
                }
            }
        }
    }
}

/// Separates schematic information that is saved to disk from information that is not saved to disk
pub struct SchematicHolder {
    /// The actual schematic, saved to disk when requested by the user
    pub schematic: Schematic,
    /// The history log for the schematic
    pub schematic_log: undo::Record<SchematicAction>,
    /// Flag that determines if the schematic has been saved
    pub schematic_was_saved: bool,
    /// The path where the schematic is saved.
    pub path: Option<StoragePath>,
    /// The file format to save the object in
    pub format: crate::general::StorageFormat,
}

impl SchematicHolder {
    /// Create an example schematicHolder
    pub fn new_example() -> Self {
        let mut rec = undo::Record::new();
        rec.set_saved(false);
        Self {
            schematic: Schematic::new_example(),
            schematic_log: rec,
            schematic_was_saved: true,
            path: None,
            format: crate::general::StorageFormat::default(),
        }
    }

    /// Retrieve the name of the schematic
    pub fn name(&self) -> String {
        if self.path.is_some() {
            self.schematic.name.clone()
        } else {
            "Unsaved schematic".to_string()
        }
    }

    /// Returns true when there are unsaved changes on the schematic object
    pub fn has_unsaved_changes(&self) -> bool {
        !self.schematic_log.is_saved()
    }

    /// True when the schematic has a path to save to
    pub fn has_path(&self) -> bool {
        self.path.is_some()
    }

    /// Set the path for the schematic when saving
    pub fn set_path(&mut self, p: StoragePath) {
        self.path = Some(p);
    }

    /// Save the schematic information to the previously configured path. Will return ok if no path is set
    pub fn save(&mut self) -> Result<(), crate::general::StorageSaveError> {
        if let Some(path) = &self.path {
            let mut writer = path.writer()?;
            return self.format.save(&mut writer, &self.schematic);
        }
        Ok(())
    }

    /// A function that determines if the status of changes made has changed. If it has, then the closure specified is run
    pub fn check_for_saved_status_change<F>(&mut self, changed: F)
    where
        F: FnOnce(&Self, bool),
    {
        let schematic_is_saved = self.schematic_log.is_saved();
        if self.schematic_was_saved != schematic_is_saved {
            self.schematic_was_saved = schematic_is_saved;
            changed(self, schematic_is_saved);
        }
    }
}

/// The things that can be selected on a schematic
pub enum SchematicSelection {
    /// A text object is selected
    Text {
        /// The page number
        page: usize,
        /// The text number
        textnum: usize,
    },
    /// A symbol has been selected
    Symbol {
        /// The page number
        page: usize,
        /// The symbol number
        sym: usize,
    },
}

/// The widget is responsible for drawing the state of the schematic for the user
pub struct SchematicWidget<'a> {
    /// The holder of the schematics
    sch: &'a mut SchematicHolder,
    /// The mouse mode for the widget
    mm: &'a mut MouseMode,
    /// The current page number that is being examined
    page: usize,
    /// The object currently selected. Will eventually become Vec<SchematicSelection>
    selection: &'a mut Option<SchematicSelection>,
    /// The origin modifier for panning the symbol around
    origin: &'a mut crate::general::Coordinates,
    /// The zoom factor
    zoom: &'a mut f32,
    /// The component that a user has selected for adding to the schematic
    component: Option<crate::component::ComponentVariantReference>,
    /// The libraries for the application
    libs: &'a HashMap<String, LibraryHolder>,
}

impl<'a> SchematicWidget<'a> {
    /// Create a new schematic widget for showing a schematic to a user
    pub fn new(
        sch: &'a mut SchematicHolder,
        mm: &'a mut MouseMode,
        sel: &'a mut Option<SchematicSelection>,
        origin: &'a mut crate::general::Coordinates,
        zoom: &'a mut f32,
        component: Option<crate::component::ComponentVariantReference>,
        libs: &'a HashMap<String, LibraryHolder>,
    ) -> Self {
        Self {
            sch,
            page: 0,
            mm,
            selection: sel,
            origin,
            zoom,
            component,
            libs,
        }
    }
}

impl<'a> egui::Widget for SchematicWidget<'a> {
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

        let (mut pr, pntr) = ui.allocate_painter(size, sense);
        let cur_page = &mut self.sch.schematic.pages[self.page];

        let zoom_origin =
            (area.left_top().to_vec2() + egui::vec2(size.x / 2.0, size.y / 2.0)).to_pos2();
        let origin = self.origin.get_pos2(*self.zoom, zoom_origin);

        //placeholder for drawing the crosshairs at the origin
        if false {
            let stroke = egui_multiwin::egui::Stroke {
                width: 1.0,
                color: egui::Color32::BLUE,
            };
            pntr.line_segment(
                [
                    egui::pos2(area.min.x, origin.y),
                    egui::pos2(area.max.x, origin.y),
                ],
                stroke,
            );
            pntr.line_segment(
                [
                    egui::pos2(origin.x, area.min.y),
                    egui::pos2(origin.x, area.max.y),
                ],
                stroke,
            );
        }

        let stroke = egui_multiwin::egui::Stroke {
            width: 1.0,
            color: Colors::Border.get_color32(crate::general::ColorMode::ScreenModeDark),
        };
        let sheet_max = cur_page.size.get_pos2(*self.zoom, origin);
        pntr.line_segment(
            [
                egui::pos2(origin.x, origin.y),
                egui::pos2(origin.x, sheet_max.y),
            ],
            stroke,
        );
        pntr.line_segment(
            [
                egui::pos2(origin.x, sheet_max.y),
                egui::pos2(sheet_max.x, sheet_max.y),
            ],
            stroke,
        );
        pntr.line_segment(
            [
                egui::pos2(sheet_max.x, origin.y),
                egui::pos2(sheet_max.x, sheet_max.y),
            ],
            stroke,
        );
        pntr.line_segment(
            [
                egui::pos2(origin.x, origin.y),
                egui::pos2(sheet_max.x, origin.y),
            ],
            stroke,
        );

        let mut actions = Vec::new();

        match &self.mm {
            MouseMode::NewText | MouseMode::TextDrag => {
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    *self.mm = MouseMode::Selection;
                }
                if self.selection.is_some() {
                    *self.selection = None;
                }
            }
            MouseMode::Selection => {
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) && self.selection.is_some() {
                    *self.selection = None;
                }
            }
            MouseMode::NewComponent => {
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    *self.mm = MouseMode::Selection;
                }
            }
        }

        if pr.clicked() && self.mm == &MouseMode::Selection {
            *self.selection = None;
        }

        if let MouseMode::NewText = &self.mm {
            let pos = ui.input(|i| i.pointer.interact_pos());
            if let Some(pos) = pos {
                if pr.clicked() {
                    let pos2 = pos - area.left_top();
                    actions.push(SchematicAction::CreateText {
                        pagenum: self.page,
                        text: TextOnPage {
                            text: "New text".to_string(),
                            location: crate::general::Coordinates::from_pos2(
                                pos2.to_pos2(),
                                *self.zoom,
                            ),
                            color: Colors::Standard,
                            size: crate::general::Length::Inches(0.2),
                        },
                    });
                } else {
                    pntr.text(
                        pos,
                        egui::Align2::LEFT_BOTTOM,
                        "New text".to_string(),
                        egui::FontId {
                            size: crate::general::Length::Inches(0.2)
                                .get_screen(*self.zoom, egui::pos2(0.0, 0.0)),
                            family: egui::FontFamily::Name("computermodern".into()),
                        },
                        Colors::Standard.get_color32(crate::general::ColorMode::ScreenModeDark),
                    );
                }
            }
        }

        for (i, t) in cur_page.texts.iter().enumerate() {
            let pos = t.location.get_pos2(*self.zoom, origin);
            let align = egui::Align2::LEFT_BOTTOM;
            let font = egui::FontId {
                size: t.size.get_screen(*self.zoom, zoom_origin),
                family: egui::FontFamily::Name("computermodern".into()),
            };
            let color = t
                .color
                .get_color32(crate::general::ColorMode::ScreenModeDark);
            let r = pntr.text(pos, align, t.text.clone(), font, color);
            let r = r.intersect(area);
            if r.is_positive() {
                let id = egui::Id::new(1 + i);
                let response = ui.interact(r.intersect(area), id, sense);
                let response = match self.mm {
                    MouseMode::NewComponent => response,
                    MouseMode::NewText => response,
                    MouseMode::Selection => {
                        if response.clicked() {
                            println!("Clicked in selection mode");
                            *self.selection = Some(SchematicSelection::Text {
                                page: self.page,
                                textnum: i,
                            });
                        }
                        let r = response.context_menu(|ui| {
                            if ui.button("Properties").clicked() {
                                ui.close_menu();
                            }
                        });
                        r.map(|r|r.response).or(Some(response)).unwrap()
                    }
                    MouseMode::TextDrag => {
                        if response.clicked() {
                            println!("Clicked in drag mode");
                        }
                        if response.dragged() {
                            let amount = response.drag_delta();
                            let a = SchematicAction::MoveText {
                                pagenum: self.page,
                                textnum: i,
                                delta: crate::general::Coordinates::from_pos2(
                                    amount.to_pos2(),
                                    *self.zoom,
                                ),
                            };
                            actions.push(a);
                        }
                        let r = response.context_menu(|ui| {
                            if ui.button("Properties").clicked() {
                                ui.close_menu();
                            }
                        });
                        r.map(|r|r.response).or(Some(response)).unwrap()
                    }
                };
                pr = pr.union(response);
            }
        }

        for a in actions {
            self.sch.schematic_log.apply(&mut self.sch.schematic, a);
        }

        let cur_page = &mut self.sch.schematic.pages[self.page];
        let mut actions = Vec::new();

        for (i, sch) in &mut cur_page.syms.iter().enumerate() {
            let mut sym = None;
            if let Some(lib) = self.libs.get(&sch.lib) {
                if let Some(lib) = &lib.library {
                    if let Some(component) = lib.components.get(&sch.com) {
                        if let Some(variant) = component.variants.get(&sch.var) {
                            if let Some(symbol) = &variant.symbol {
                                sym = Some((symbol, lib));
                            }
                        }
                    }
                }
            }
            if let Some((sym, lib)) = sym {
                let libname = sym.lib.get_name(&lib);
                if let Some(lib) = self.libs.get(&libname) {
                    if let Some(lib) = &lib.library {
                        if let Some(symbol) = lib.syms.get(&sym.sym) {
                            let pos = sch.pos.get_pos2(*self.zoom, origin) - zoom_origin.to_vec2();
                            let rects = symbol.draw(*self.zoom, zoom_origin, &pntr, pos, area);
                            let response =
                                crate::general::respond(ui, format!("symbol{}", i), rects);
                            let response = match &self.mm {
                                MouseMode::Selection => {
                                    if response.clicked() {
                                        *self.selection = Some(SchematicSelection::Symbol {
                                            page: self.page,
                                            sym: i,
                                        });
                                    }
                                    response
                                }
                                MouseMode::TextDrag => response,
                                MouseMode::NewText => response,
                                MouseMode::NewComponent => response,
                            };
                            pr = pr.union(response);
                        }
                    }
                }
            }
        }

        for a in actions {
            self.sch.schematic_log.apply(&mut self.sch.schematic, a);
        }

        let mut actions = Vec::new();
        if let MouseMode::NewComponent = &self.mm {
            let pos = ui.input(|i| i.pointer.interact_pos());
            if let Some(pos) = pos {
                if let Some(variantref) = self.component {
                    let sd = variantref.get_symbol(self.libs);
                    if let Some(symdef) = sd {
                        let pos2 = (pos - zoom_origin).to_pos2();
                        if pr.clicked() {
                            let mut vr = variantref.clone();
                            vr.pos = crate::general::Coordinates::from_pos2(
                                (pos - origin).to_pos2(),
                                *self.zoom,
                            );
                            actions.push(SchematicAction::AddComponentVariant {
                                pagenum: self.page,
                                var: vr,
                            });
                        } else {
                            symdef.draw(*self.zoom, zoom_origin, &pntr, pos2, area);
                        }
                    }
                }
            }
        }
        for a in actions {
            self.sch.schematic_log.apply(&mut self.sch.schematic, a);
        }

        let ipr = pr.context_menu(|ui| {
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

        pr
    }
}
