pub struct Symbol {
    texts: Vec<TextOnPage>,
}

pub struct TextOnPage {
    text: String,
    x: f32,
    y: f32,
}

pub struct Page {
    syms: Vec<Symbol>,
    texts: Vec<TextOnPage>,
}

pub struct Schematic {
    pages: Vec<Page>,
    pub mm: MouseMode,
}

/// Defines the mode for mouse interaction
#[derive(PartialEq)]
pub enum MouseMode {
    Selection,
    TextDrag,
}

pub struct SchematicWidget<'a> {
    sch: &'a mut Schematic,
    page: usize,
}

impl Schematic {
    pub fn new() -> Self {
        let mut p = Vec::new();
        let mut t = Vec::new();
        t.push(TextOnPage {
            text: "demo text".to_string(),
            x: 0.0,
            y: 0.0,
        });
        t.push(TextOnPage {
            text: "moredemo text".to_string(),
            x: 50.0,
            y: 50.0,
        });
        let page = Page {
            syms: Vec::new(),
            texts: t,
        };
        p.push(page);
        Self {
            pages: p,
            mm: MouseMode::Selection,
        }
    }
}

impl<'a> SchematicWidget<'a> {
    pub fn new(sch: &'a mut Schematic) -> Self {
        Self { sch: sch, page: 0 }
    }
}

impl<'a> eframe::egui::Widget for SchematicWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let sense = eframe::egui::Sense {
            click: true,
            drag: true,
            focusable: true,
        };
        let context = ui.ctx();
        let mut area = ui.cursor();
        area.max.x = ui.available_width() + area.min.x;
        area.max.y = ui.available_height() + area.min.y;
        let size = eframe::egui::vec2(area.max.x - area.min.x, area.max.y - area.min.y);

        let pntr = ui.painter().with_clip_rect(area);
        let cur_page = &mut self.sch.pages[self.page];
        let color = eframe::egui::Color32::RED;

        for (i, t) in cur_page.texts.iter_mut().enumerate() {
            let pos = eframe::egui::Vec2 { x: t.x, y: t.y };
            let align = eframe::egui::Align2::LEFT_TOP;
            let font = eframe::egui::FontId {
                size: 24.0,
                family: eframe::egui::FontFamily::Monospace,
            };
            let temp = area.left_top() + pos;
            let mut r = pntr.text(temp, align, t.text.clone(), font, color);
            let response = ui.interact(
                r,
                eframe::egui::Id::new(42424242 + i),
                eframe::egui::Sense {
                    click: true,
                    drag: true,
                    focusable: true,
                },
            );
            match self.sch.mm {
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
                        t.x += amount.x;
                        t.y += amount.y;
                    }
                }
            }
        }
        for mut sch in &mut cur_page.syms {
            for (i, t) in sch.texts.iter_mut().enumerate() {
                let pos = eframe::egui::Vec2 { x: t.x, y: t.y };
                let align = eframe::egui::Align2::LEFT_TOP;
                let font = eframe::egui::FontId {
                    size: 24.0,
                    family: eframe::egui::FontFamily::Monospace,
                };
                let temp = area.left_top() + pos;
                let mut r = pntr.text(temp, align, t.text.clone(), font, color);
                let response = ui.interact(
                    r,
                    eframe::egui::Id::new(42424242 + i),
                    eframe::egui::Sense {
                        click: true,
                        drag: true,
                        focusable: true,
                    },
                );
                match self.sch.mm {
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
                            t.x += amount.x;
                            t.y += amount.y;
                        }
                    }
                }
            }
        }

        pntr.rect_stroke(
            area,
            eframe::egui::Rounding {
                nw: 5.0,
                ne: 5.0,
                sw: 5.0,
                se: 5.0,
            },
            eframe::egui::Stroke {
                width: 5.0,
                color: color,
            },
        );
        let (area, response) = ui.allocate_exact_size(
            size,
            eframe::egui::Sense {
                click: true,
                drag: true,
                focusable: true,
            },
        );
        response
    }
}
