use eframe::egui;
use egui::style::Visuals;
use egui::Context;

use crate::config::{Config, Theme};

// Sets up visuals and styles
pub fn setup_ui(config: &Config, ctx: &Context) {
    let visuals = match config.theme {
        Theme::Light => Visuals::light(),
        Theme::Dark => Visuals::dark(),
    };
    ctx.set_visuals(visuals)
}
