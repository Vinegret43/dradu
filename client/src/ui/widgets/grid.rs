use eframe::egui;
use egui::{Color32, Pos2, Shape, Stroke, Ui};

const STROKE_WIDTH: f32 = 1.0;
const STROKE_COLOR: Color32 = Color32::GRAY;

pub fn draw_grid(size: [u8; 2], ui: &Ui) {
    let mut shapes = Vec::new();
    let stroke = Stroke::new(STROKE_WIDTH, STROKE_COLOR);
    let rect = ui.min_rect();
    // Drawing vertical lines
    let offset = rect.width() / size[0] as f32;
    for i in 1..size[0] {
        let x = rect.min.x + offset * i as f32;
        shapes.push(Shape::line_segment(
            [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
            stroke,
        ));
    }
    // Drawing horizontal lines
    let offset = rect.height() / size[1] as f32;
    for i in 1..size[1] {
        let y = rect.min.y + offset * i as f32;
        shapes.push(Shape::line_segment(
            [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
            stroke,
        ));
    }
    let painter = ui.painter();
    painter.extend(shapes);
}
