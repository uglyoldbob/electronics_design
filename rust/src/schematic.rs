pub struct Symbol {

}

pub struct Schematic {

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
        
        response
    }
}