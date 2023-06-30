//! The schematic module covers code related to a electronics schematic, consisting of one or more pages of stuff.

use egui_multiwin::egui;

use std::io::Write;

use crate::symbol::Symbol;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
/// Represents free text anywhere on a page or symbol
pub struct TextOnPage {
    /// The text to display
    pub text: String,
    /// The x location of the text
    pub x: f32,
    /// The y location of the text
    pub y: f32,
    /// The color for the text. See [egui::Color32] for the `to_srgba_unmultiplied` function
    pub color: [u8; 4],
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
/// A single page of an electronic schematic
pub struct Page {
    /// The symbols on the page
    pub syms: Vec<Symbol>,
    /// The free text items on the page
    pub texts: Vec<TextOnPage>,
}

#[derive(serde::Serialize, serde::Deserialize)]
/// Represents an entire electronic schematic
pub struct Schematic {
    /// The list of pages for the schematic
    pub pages: Vec<Page>,
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
                x: 0.0,
                y: 0.0,
                color: egui_multiwin::egui::Color32::RED.to_srgba_unmultiplied(),
            },
            TextOnPage {
                text: "moredemo text".to_string(),
                x: 50.0,
                y: 50.0,
                color: egui_multiwin::egui::Color32::BLUE.to_srgba_unmultiplied(),
            },
        ];
        let page = Page {
            syms: Vec::new(),
            texts: t,
        };
        p.push(page);
        Self { pages: p }
    }

    /// Save the schematic to the path specified, returns true when the saving occurred
    pub fn save(&self, path: &String) -> Result<(), std::io::Error> {
        let d = bincode::serialize(self).unwrap();
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)?;
        file.write_all(&d)?;
        Ok(())
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
        /// The delta x to move by
        dx: f32,
        /// The delta y to move by
        dy: f32,
    },
    /// Move the text of a symbol on a schematic page by a certain amount
    MoveSymbolText {
        /// The page number
        pagenum: usize,
        /// The symbol number
        symbolnum: usize,
        /// The text number
        textnum: usize,
        /// The delta x to move by
        dx: f32,
        /// The delta y to move by
        dy: f32,
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
                dx,
                dy,
            } => {
                target.pages[*pagenum].texts[*textnum].x += *dx;
                target.pages[*pagenum].texts[*textnum].y += *dy;
            }
            SchematicAction::MoveSymbolText {
                pagenum,
                symbolnum,
                textnum,
                dx,
                dy,
            } => {
                target.pages[*pagenum].syms[*symbolnum].texts[*textnum].x += *dx;
                target.pages[*pagenum].syms[*symbolnum].texts[*textnum].y += *dy;
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
                dx,
                dy,
            } => {
                target.pages[*pagenum].texts[*textnum].x -= *dx;
                target.pages[*pagenum].texts[*textnum].y -= *dy;
            }
            SchematicAction::MoveSymbolText {
                pagenum,
                symbolnum,
                textnum,
                dx,
                dy,
            } => {
                target.pages[*pagenum].syms[*symbolnum].texts[*textnum].x -= *dx;
                target.pages[*pagenum].syms[*symbolnum].texts[*textnum].y -= *dy;
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
                dx,
                dy,
            } => {
                if let SchematicAction::MoveText {
                    pagenum: pn2,
                    textnum: tn2,
                    dx: dx2,
                    dy: dy2,
                } = other.clone()
                {
                    if *pagenum == pn2 && *textnum == tn2 {
                        if (*dx + dx2) < f32::EPSILON && (*dy + dy2) < f32::EPSILON {
                            undo::Merged::Annul
                        } else {
                            *dx += dx2;
                            *dy += dy2;
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
                dx,
                dy,
            } => {
                if let SchematicAction::MoveSymbolText {
                    pagenum: pn2,
                    symbolnum: sn2,
                    textnum: tn2,
                    dx: dx2,
                    dy: dy2,
                } = other
                {
                    if *pagenum == pn2 && *symbolnum == sn2 && *textnum == tn2 {
                        if (*dx + dx2) < f32::EPSILON && (*dy + dy2) < f32::EPSILON {
                            undo::Merged::Annul
                        } else {
                            *dx += dx2;
                            *dy += dy2;
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
    /// The path where the schematic is saved. This may become Option<PathEnum> in the future, where PathEnum defines the many options/places to save data
    path: Option<String>,
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
        }
    }

    /// Retrieve the name of the schematic
    pub fn name(&self) -> String {
        if let Some(p) = &self.path {
            p.to_owned()
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
    pub fn set_path(&mut self, p: String) {
        self.path = Some(p);
    }

    /// Save the schematic information to the previously configured path
    pub fn save(&mut self) -> Result<(), std::io::Error> {
        let e = self.schematic.save(self.path.as_ref().unwrap());
        if e.is_ok() {
            self.schematic_log.set_saved(true);
        }
        e
    }

    /// Load a schematic from the specified location
    pub fn load(path: String, buffer: &[u8]) -> Option<Self> {
        let sch = bincode::deserialize::<Schematic>(buffer);
        if let Ok(sch) = sch {
            let rec = undo::Record::new();
            Some(Self {
                schematic: sch,
                schematic_log: rec,
                schematic_was_saved: false,
                path: Some(path),
            })
        } else {
            None
        }
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
}

impl<'a> SchematicWidget<'a> {
    /// Create a new schematic widget for showing a schematic to a user
    pub fn new(
        sch: &'a mut SchematicHolder,
        mm: &'a mut MouseMode,
        sel: &'a mut Option<SchematicSelection>,
    ) -> Self {
        Self {
            sch,
            page: 0,
            mm,
            selection: sel,
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

        let (pr, pntr) = ui.allocate_painter(size, sense);
        let cur_page = &mut self.sch.schematic.pages[self.page];
        let color = egui::Color32::RED;

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

        if let MouseMode::NewText = &self.mm {
            let pos = ui.input(|i| i.pointer.interact_pos());
            if let Some(pos) = pos {
                if pr.clicked() {
                    let pos2 = pos - area.left_top();
                    actions.push(SchematicAction::CreateText {
                        pagenum: self.page,
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

        for (i, t) in cur_page.texts.iter().enumerate() {
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
                        *self.selection = Some(SchematicSelection::Text {
                            page: self.page,
                            textnum: i,
                        });
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
                        let a = SchematicAction::MoveText {
                            pagenum: self.page,
                            textnum: i,
                            dx: amount.x,
                            dy: amount.y,
                        };
                        actions.push(a);
                    }
                    response.context_menu(|ui| {
                        if ui.button("Properties").clicked() {
                            ui.close_menu();
                        }
                    });
                }
            }
        }

        for a in actions {
            self.sch.schematic_log.apply(&mut self.sch.schematic, a);
        }

        let cur_page = &mut self.sch.schematic.pages[self.page];
        let mut actions = Vec::new();

        for sch in &mut cur_page.syms {
            for (i, t) in sch.texts.iter().enumerate() {
                let pos = egui::Vec2 { x: t.x, y: t.y };
                let align = egui::Align2::LEFT_TOP;
                let font = egui::FontId {
                    size: 24.0,
                    family: egui::FontFamily::Monospace,
                };
                let temp = area.left_top() + pos;
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
                                dx: amount.x,
                                dy: amount.y,
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

        if pr.clicked() {
            match self.mm {
                MouseMode::Selection => {
                    *self.selection = None;
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
