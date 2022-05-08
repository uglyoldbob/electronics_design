pub struct Symbol {}

pub struct Page {
    syms: Vec<Symbol>,
}

pub struct Schematic {
    pages: Vec<Page>,
}

impl Schematic {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }
}

impl eframe::egui::Widget for Schematic {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let size = eframe::egui::vec2(100.0, 100.0);
        let sense = eframe::egui::Sense {
            click: true,
            drag: true,
            focusable: true,
        };
        let (area, response) = ui.allocate_at_least(size, sense);
        let pntr = ui.painter().with_clip_rect(area);
        let pos = eframe::egui::Pos2 { x: 0.0, y: 0.0 };
        let align = eframe::egui::Align2::LEFT_TOP;
        let font = eframe::egui::FontId {
            size: 24.0,
            family: eframe::egui::FontFamily::Monospace,
        };
        let color = eframe::egui::Color32::RED;
        pntr.text(area.left_top(), align, "testing fjdklsaksjd fjkdlaskjd fjdklsasdf", font, color);
        pntr.rect_stroke(area, eframe::egui::Rounding{
            nw: 5.0, ne: 5.0, sw: 5.0, se: 5.0,
        }, eframe::egui::Stroke{width: 5.0, color: color});
        response
    }
}
