#![windows_subsystem = "windows"]

mod config;
mod error;
mod fs;
mod net;
mod setup;
mod state;
mod textures;
mod ui;
mod utils;

pub use crate::error::DraduError;

use eframe::egui;
use eframe::{Frame, Storage};
use egui::Context;

use crate::config::Config;
use crate::state::RoomState;
use crate::textures::Textures;
use crate::ui::{MainUi, MenuAction, MenuUi};

struct DraduApp {
    config: Config,
    textures: Textures,
    main_ui: MainUi,
    menu_ui: MenuUi,
    room_state: Option<RoomState>,
}

impl DraduApp {
    // This function will set everything up, including egui and the app itself:
    // load settings, set up theming and fonts, check integrity of filesystem
    fn new(cc: &eframe::CreationContext) -> Self {
        let config = Config::load(cc.storage.unwrap());
        setup::setup_ui(&config, &cc.egui_ctx);

        let textures = Textures::new(&cc.egui_ctx);
        let main_ui = MainUi::new(textures.clone());
        let menu_ui = MenuUi::new(textures.clone(), &config);

        DraduApp {
            config,
            textures,
            main_ui,
            menu_ui,
            room_state: None,
        }
    }

    // Called after player left the room (Disconnected)
    // FIXME: add ability to reconnect again without loosing all the data
    fn reset(&mut self) {
        self.main_ui = MainUi::new(self.textures.clone());
        self.menu_ui = MenuUi::new(self.textures.clone(), &self.config);
        self.room_state = None;
    }

    fn set_nickname_and_color(config: &Config, state: &mut RoomState) {
        if !config.nickname.trim().is_empty() {
            state.send_chat_message(&format!("/nickname {}", config.nickname));
        }
        if config.custom_color_enabled {
            state.send_chat_message(&format!("/color {}", utils::rgb_to_string(config.color)));
        }
    }
}

impl eframe::App for DraduApp {
    // This func reroutes execution to some other UI, based on circumstances
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        match &mut self.room_state {
            Some(ref mut state) => {
                if let Err(_) = state.update_self() {
                    self.reset();
                    return;
                }
                if let Err(_) = self.main_ui.update(ctx, state) {
                    self.reset();
                }
            }
            state => match self.menu_ui.update(ctx, frame, &mut self.config) {
                MenuAction::JoinRoom(addr, room_id) => {
                    *state = RoomState::join_room(addr, &room_id, ctx).ok();
                    if let Some(s) = state {
                        Self::set_nickname_and_color(&self.config, s);
                    }
                }
                MenuAction::NewRoom(addr) => {
                    *state = RoomState::create_new_room(addr, ctx).ok();
                    if let Some(s) = state {
                        Self::set_nickname_and_color(&self.config, s);
                    }
                }
                MenuAction::MapCreator => {
                    *state = Some(RoomState::create_local_server(ctx));
                }
                MenuAction::None => (),
            },
        }
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        self.config.save(storage);
    }
}

fn main() {
    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        ..Default::default()
    };
    eframe::run_native("Dradu", options, Box::new(|cc| Box::new(DraduApp::new(cc))));
}
