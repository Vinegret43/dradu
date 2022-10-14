use eframe::egui;
use egui::containers::CentralPanel;
use egui::widgets::Spinner;
use egui::Context;

pub struct LoadingScreenUi {}

impl LoadingScreenUi {
    pub fn new() -> Self {
        LoadingScreenUi {}
    }
}

impl LoadingScreenUi {
    pub fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 2.0 - 20.0);
                ui.heading("Connecting to the server...");
                ui.add(Spinner::new());
            });
        });
    }
}
