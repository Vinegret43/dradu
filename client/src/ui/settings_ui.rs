use eframe::egui;
use egui::containers::CentralPanel;
use egui::{Context, Visuals};

use crate::config::{Config, Theme};

#[derive(Default)]
pub struct SettingsUi {
    pub is_opened: bool,
    color: [u8; 3],
}

impl SettingsUi {
    pub fn new(config: &Config) -> Self {
        Self {
            is_opened: false,
            color: config.color,
        }
    }

    pub fn update(&mut self, ctx: &Context, config: &mut Config) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("App Theme");
            if ui
                .radio_value(&mut config.theme, Theme::Dark, "Dark")
                .changed()
            {
                ctx.set_visuals(Visuals::dark())
            };
            if ui
                .radio_value(&mut config.theme, Theme::Light, "Light")
                .changed()
            {
                ctx.set_visuals(Visuals::light())
            };

            ui.label("Nickname");
            ui.text_edit_singleline(&mut config.nickname);

            ui.checkbox(&mut config.custom_color_enabled, "Custom color enabled");

            ui.label("Color");
            if ui.color_edit_button_srgb(&mut self.color).changed() {
                config.color = self.color;
            }

            if ui.button("Back to menu").clicked() {
                self.is_opened = false;
            }
        });
    }
}
