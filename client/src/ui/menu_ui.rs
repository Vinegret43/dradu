use eframe::egui;
use egui::containers::CentralPanel;

use egui::Context;

use std::net::SocketAddr;

use crate::config::Config;
use crate::textures::Textures;
use crate::ui::SettingsUi;

pub struct MenuUi {
    textures: Textures,
    join_addr: String,
    new_game_addr: String,
    settings_ui: SettingsUi,
}

impl MenuUi {
    pub fn new(textures: Textures, config: &Config) -> Self {
        MenuUi {
            textures,
            join_addr: String::new(),
            new_game_addr: String::new(),
            settings_ui: SettingsUi::new(config),
        }
    }
}

impl MenuUi {
    pub fn update(
        &mut self,
        ctx: &Context,
        frame: &mut eframe::Frame,
        mut config: &mut Config,
    ) -> MenuAction {
        if self.settings_ui.is_opened {
            self.settings_ui.update(ctx, &mut config);
            return MenuAction::None;
        }
        CentralPanel::default()
            .show(ctx, |ui| {
                ui.heading("DRADU");

                let mut response = MenuAction::None;

                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.join_addr);
                    if ui.button("Join").clicked() {
                        if let Some((addr, room_id)) = self.join_addr.split_once('#') {
                            if let Ok(addr) = addr.parse::<SocketAddr>() {
                                response = MenuAction::JoinRoom(addr, room_id.to_string());
                            }
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.new_game_addr);
                    if ui.button("New game").clicked() {
                        let server_address = if self.new_game_addr.contains(':') {
                            self.new_game_addr.parse::<SocketAddr>()
                        } else {
                            format!("{}:8889", self.new_game_addr).parse::<SocketAddr>()
                        };
                        if let Ok(addr) = server_address {
                            response = MenuAction::NewRoom(addr);
                        }
                    }
                });
                if ui.button("Map creator").clicked() {
                    response = MenuAction::MapCreator;
                }
                if ui.button("Settings").clicked() {
                    self.settings_ui.is_opened = true;
                }
                if ui.button("Quit").clicked() {
                    frame.close()
                }
                response
            })
            .inner
    }
}

pub enum MenuAction {
    JoinRoom(SocketAddr, String),
    NewRoom(SocketAddr),
    MapCreator,
    None,
}
