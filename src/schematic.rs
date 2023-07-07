//! The schematic module covers code related to a electronics schematic, consisting of one or more pages of stuff.

use egui_multiwin::egui;

use crate::{general::StoragePath, symbol::Symbol};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[non_exhaustive]
/// Represents free text anywhere on a page or symbol
pub struct TextOnPage {
    /// The text to display
    pub text: String,
    /// The location of the text
    pub location: crate::general::Coordinates,
    /// The color for the text. See [egui::Color32] for the `to_srgba_unmultiplied` function
    pub color: [u8; 4],
    /// The size of the text in physical dimensions
    pub size: crate::general::Length,
}

impl TextOnPage {
    /// Builds a color for the text, merely a convenience function
    pub fn color(&self) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(
            self.color[0],
            self.color[1],
            self.color[2],
            self.color[3],
        )
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
/// A single page of an electronic schematic
pub struct Page {
    /// The symbols on the page
    pub syms: Vec<Symbol>,
    /// The free text items on the page
    pub texts: Vec<TextOnPage>,
}

impl Page {
    /// Draw the page on the given pdf layer
    pub fn draw_on(&self, layer: printpdf::PdfLayerReference) {
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
            has_fill: true,
            has_stroke: true,
            is_clipping_path: false,
        };
        layer.add_shape(line);
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
}

impl Schematic {
    /// Create a new example schematic.
    pub fn new_example() -> Self {
        let mut p = Vec::new();
        let t = vec![
            TextOnPage {
                text: "demo text".to_string(),
                location: crate::general::Coordinates::Inches(0.0, 0.0),
                color: egui_multiwin::egui::Color32::RED.to_srgba_unmultiplied(),
                size: crate::general::Length::Inches(0.2),
            },
            TextOnPage {
                text: "moredemo text".to_string(),
                location: crate::general::Coordinates::Inches(1.0, 1.0),
                color: egui_multiwin::egui::Color32::BLUE.to_srgba_unmultiplied(),
                size: crate::general::Length::Inches(0.2),
            },
        ];
        let page = Page {
            syms: Vec::new(),
            texts: t,
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
    /// Move text on a schematic page by a certain amount
    MoveText {
        /// The page number
        pagenum: usize,
        /// The text number
        textnum: usize,
        /// The delta to move by
        delta: crate::general::Coordinates,
    },
    /// Move the text of a symbol on a schematic page by a certain amount
    MoveSymbolText {
        /// The page number
        pagenum: usize,
        /// The symbol number
        symbolnum: usize,
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
        old: egui::Color32,
        /// The new text
        new: egui::Color32,
    },
}

impl undo::Action for SchematicAction {
    type Target = Schematic;

    type Output = ();

    fn apply(&mut self, target: &mut Self::Target) -> Self::Output {
        match self {
            SchematicAction::MoveText {
                pagenum,
                textnum,
                delta,
            } => {
                target.pages[*pagenum].texts[*textnum].location += *delta;
            }
            SchematicAction::MoveSymbolText {
                pagenum,
                symbolnum,
                textnum,
                delta,
            } => {
                target.pages[*pagenum].syms[*symbolnum].texts[*textnum].location += *delta;
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
                target.pages[*pagenum].texts[*textnum].color = new.to_srgba_unmultiplied();
            }
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        match self {
            SchematicAction::MoveText {
                pagenum,
                textnum,
                delta,
            } => {
                target.pages[*pagenum].texts[*textnum].location -= *delta;
            }
            SchematicAction::MoveSymbolText {
                pagenum,
                symbolnum,
                textnum,
                delta,
            } => {
                target.pages[*pagenum].syms[*symbolnum].texts[*textnum].location -= *delta;
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
                target.pages[*pagenum].texts[*textnum].color = old.to_srgba_unmultiplied();
            }
        }
    }

    fn merge(&mut self, other: Self) -> undo::Merged<Self>
    where
        Self: Sized,
    {
        match self {
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
            SchematicAction::MoveSymbolText {
                pagenum,
                symbolnum,
                textnum,
                delta,
            } => {
                if let SchematicAction::MoveSymbolText {
                    pagenum: pn2,
                    symbolnum: sn2,
                    textnum: tn2,
                    delta: delta2,
                } = other
                {
                    if *pagenum == pn2 && *symbolnum == sn2 && *textnum == tn2 {
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
            return Ok(self.format.save(&mut writer, &self.schematic)?);
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
}

impl<'a> SchematicWidget<'a> {
    /// Create a new schematic widget for showing a schematic to a user
    pub fn new(
        sch: &'a mut SchematicHolder,
        mm: &'a mut MouseMode,
        sel: &'a mut Option<SchematicSelection>,
        origin: &'a mut crate::general::Coordinates,
        zoom: &'a mut f32,
    ) -> Self {
        Self {
            sch,
            page: 0,
            mm,
            selection: sel,
            origin,
            zoom,
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
        let color = egui::Color32::RED;

        let zoom_origin =
            (area.left_top().to_vec2() + egui::vec2(size.x / 2.0, size.y / 2.0)).to_pos2();
        let origin = self.origin.get_pos2(*self.zoom, zoom_origin);

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
        }

        if pr.clicked() {
            match self.mm {
                MouseMode::Selection => {
                    *self.selection = None;
                }
                _ => {}
            }
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
                            size: crate::general::Length::Inches(0.2)
                                .get_screen(*self.zoom, egui::pos2(0.0, 0.0)),
                            family: egui::FontFamily::Monospace,
                        },
                        color,
                    );
                }
            }
        }

        for (i, t) in cur_page.texts.iter().enumerate() {
            let pos = t.location.get_pos2(*self.zoom, origin);
            let align = egui::Align2::LEFT_TOP;
            let font = egui::FontId {
                size: t.size.get_screen(*self.zoom, zoom_origin),
                family: egui::FontFamily::Monospace,
            };
            let color = t.color();
            let r = pntr.text(pos, align, t.text.clone(), font, color);
            let id = egui::Id::new(1 + i);
            let response = ui.interact(r, id, sense);
            let response = match self.mm {
                MouseMode::NewText => response,
                MouseMode::Selection => {
                    if response.clicked() {
                        println!("Clicked in selection mode");
                        *self.selection = Some(SchematicSelection::Text {
                            page: self.page,
                            textnum: i,
                        });
                    }
                    response.context_menu(|ui| {
                        if ui.button("Properties").clicked() {
                            ui.close_menu();
                        }
                    })
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
                    response.context_menu(|ui| {
                        if ui.button("Properties").clicked() {
                            ui.close_menu();
                        }
                    })
                }
            };
            pr = pr.union(response);
        }

        for a in actions {
            self.sch.schematic_log.apply(&mut self.sch.schematic, a);
        }

        let cur_page = &mut self.sch.schematic.pages[self.page];
        let mut actions = Vec::new();

        for sch in &mut cur_page.syms {
            for (i, t) in sch.texts.iter().enumerate() {
                let pos = t.location.get_pos2(*self.zoom, egui::pos2(0.0, 0.0));
                let align = egui::Align2::LEFT_TOP;
                let font = egui::FontId {
                    size: 24.0,
                    family: egui::FontFamily::Monospace,
                };
                let temp = area.left_top() + pos.to_vec2();
                let color = t.color();
                let r = pntr.text(temp, align, t.text.clone(), font, color);
                let response = ui.interact(r, egui::Id::new(42424242 + i), sense);
                match self.mm {
                    MouseMode::NewText => {}
                    MouseMode::Selection => {
                        if response.clicked() {
                            println!("Clicked");
                        }
                    }
                    MouseMode::TextDrag => {
                        if response.clicked() {
                            println!("Clicked");
                        }
                        if response.dragged() {
                            let amount = response.drag_delta();
                            let a = SchematicAction::MoveSymbolText {
                                pagenum: self.page,
                                symbolnum: i,
                                textnum: i,
                                delta: crate::general::Coordinates::from_pos2(
                                    amount.to_pos2(),
                                    *self.zoom,
                                ),
                            };
                            actions.push(a);
                        }
                    }
                }
            }
        }

        for a in actions {
            self.sch.schematic_log.apply(&mut self.sch.schematic, a);
        }

        let pr = pr.context_menu(|ui| {
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
        pr.union(response)
    }
}
