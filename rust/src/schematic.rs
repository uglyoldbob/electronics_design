pub struct Symbol {}

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
}

pub struct SchematicWidget<'a> {
    sch: &'a Schematic,
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
        let page = Page {
            syms: Vec::new(),
            texts: t,
        };
        p.push(page);
        Self { pages: p }
    }
}

impl<'a> SchematicWidget<'a> {
    pub fn new(sch: &'a Schematic) -> Self {
        Self { sch: sch, page: 0 }
    }
}

impl<'a> eframe::egui::Widget for SchematicWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let size = eframe::egui::vec2(100.0, 100.0);
        let sense = eframe::egui::Sense {
            click: true,
            drag: true,
            focusable: true,
        };
        let (area, response) = ui.allocate_at_least(size, sense);
        let pntr = ui.painter().with_clip_rect(area);
        let cur_page = &self.sch.pages[self.page];
        let color = eframe::egui::Color32::RED;
        for t in &cur_page.texts {
            let pos = eframe::egui::Pos2 { x: t.x, y: t.y };
            let align = eframe::egui::Align2::LEFT_TOP;
            let font = eframe::egui::FontId {
                size: 24.0,
                family: eframe::egui::FontFamily::Monospace,
            };
            pntr.text(area.left_top(), align, t.text.clone(), font, color);
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
        response
    }
}
