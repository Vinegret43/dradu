use eframe::Storage;

use crate::utils;

pub struct Config {
    pub theme: Theme,
    pub nickname: String,
    pub custom_color_enabled: bool,
    pub color: [u8; 3],
}

impl Config {
    pub fn load(storage: &dyn Storage) -> Self {
        let theme = match storage.get_string("theme") {
            Some(s) if &s.to_lowercase() == "light" => Theme::Light,
            _ => Theme::Dark,
        };

        let custom_color_enabled = match storage.get_string("custom_color_enabled") {
            Some(s) if &s.to_lowercase() == "true" => true,
            _ => false,
        };

        let nickname = storage
            .get_string("nickname")
            .unwrap_or(String::new())
            .trim()
            .to_owned();

        let color = utils::parse_color(
            &storage
                .get_string("color")
                .unwrap_or("255 255 255".to_string()),
        )
        .unwrap_or([255, 255, 255]);

        Self {
            theme,
            custom_color_enabled,
            nickname,
            color,
        }
    }

    pub fn save(&self, storage: &mut dyn Storage) {
        if !self.nickname.trim().is_empty() {
            storage.set_string("nickname", self.nickname.clone());
        }
        storage.set_string("color", utils::rgb_to_string(self.color));
        storage.set_string(
            "custom_color_enabled",
            self.custom_color_enabled.to_string(),
        );
        storage.set_string("theme", self.theme.to_string());
    }
}

#[derive(strum_macros::Display, PartialEq, Clone, Copy)]
pub enum Theme {
    Dark,
    Light,
}
