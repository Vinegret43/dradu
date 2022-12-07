use eframe::egui;
use egui::containers::ScrollArea;
use egui::{Align, Context, Layout, Ui};

use json::JsonValue;

use std::fs::{self, File, ReadDir};
use std::path::{Path, PathBuf};

use crate::net::{Message, MsgBody, MsgType};
use crate::state::{map::MapState, RoomState};
use crate::ui::Window;
use crate::utils;

pub struct MapManager {
    save_map_input: String,
    map_delete_confirm: Confirm,
    save_overwrite_confirm: Confirm,
    map_fs_handler: Option<MapHandler>,
}

impl Default for MapManager {
    fn default() -> Self {
        Self {
            save_map_input: String::new(),
            map_delete_confirm: Confirm::None,
            save_overwrite_confirm: Confirm::None,
            map_fs_handler: MapHandler::new(),
        }
    }
}

impl MapManager {
    fn display_map_loader(&mut self, ui: &mut Ui, room_state: &mut RoomState) {
        if let Some(map_handler) = &self.map_fs_handler {
            ui.label("Load/save map");
            ScrollArea::vertical().show(ui, |ui| {
                if let Ok(map_list) = map_handler.list_maps() {
                    for fname in map_list {
                        ui.horizontal(|ui| {
                            if let Some(stem) = fname.unwrap().path().file_stem() {
                                ui.label(stem.to_str().unwrap_or("Error"));
                                if ui.button("Load").clicked() {
                                    map_handler.load_map(&stem, room_state);
                                }
                            }
                        });
                    }
                }
            });

            ui.add_space(10.0);

            // When just using ui.horizontal text input will make the window width
            // grow to an insane value. Fixing this with left_to_right layout
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                match self.save_overwrite_confirm {
                    Confirm::None => {
                        if ui.button("Save").clicked() {
                            if map_handler.map_exists(&self.save_map_input) {
                                self.save_overwrite_confirm = Confirm::Requested;
                            } else {
                                self.save_overwrite_confirm = Confirm::Confirmed;
                            }
                        }
                    }
                    Confirm::Requested => {
                        if ui.button("X").clicked() {
                            self.save_overwrite_confirm = Confirm::None;
                        }
                        if ui.button("✔").clicked() {
                            self.save_overwrite_confirm = Confirm::Confirmed;
                        }
                        ui.label("Overwrite?");
                    }
                    Confirm::Confirmed => {
                        map_handler.save_map(&self.save_map_input, room_state.map());
                        self.save_overwrite_confirm = Confirm::None;
                        self.save_map_input = String::new();
                    }
                }
                ui.text_edit_singleline(&mut self.save_map_input);
            });
        } else {
            ui.label("Map saving is not available");
        }
    }

    fn display_map_controls(&mut self, ui: &mut Ui, room_state: &mut RoomState) {
        ui.label("Map actions");
        match self.map_delete_confirm {
            Confirm::None => {
                ui.horizontal(|ui| {
                    if ui.button("Clear map").clicked() {
                        self.map_delete_confirm = Confirm::Requested;
                    }
                });
            }
            Confirm::Requested => {
                ui.label("Are you sure?");
                ui.horizontal(|ui| {
                    if ui.button("✔").clicked() {
                        self.map_delete_confirm = Confirm::Confirmed;
                    }
                    if ui.button("X").clicked() {
                        self.map_delete_confirm = Confirm::None;
                    }
                });
            }
            Confirm::Confirmed => {
                room_state.clear_map();
                self.map_delete_confirm = Confirm::None;
            }
        }
    }
}

impl Window for MapManager {
    fn show(&mut self, ctx: &Context, room_state: &mut RoomState) -> bool {
        if !room_state.is_master() {
            return false;
        }
        let mut open = true;
        eframe::egui::Window::new("Map Manager")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.columns(2, |cols| {
                    self.display_map_loader(&mut cols[0], room_state);

                    self.display_map_controls(&mut cols[1], room_state);
                });
            });
        open
    }
}

enum Confirm {
    None,
    Requested,
    Confirmed,
}

impl Default for Confirm {
    fn default() -> Self {
        Self::None
    }
}

struct MapHandler {
    map_dir: PathBuf,
}

impl MapHandler {
    fn new() -> Option<Self> {
        match utils::local_dir() {
            Some(mut path) => {
                path.push("maps");
                if !path.exists() {
                    #[allow(unused)]
                    {
                        std::fs::create_dir_all(&path);
                    }
                }
                Some(Self { map_dir: path })
            }
            None => None,
        }
    }

    fn list_maps(&self) -> std::io::Result<ReadDir> {
        std::fs::read_dir(&self.map_dir)
    }

    fn map_exists(&self, name: &str) -> bool {
        let path = self.get_map_path_by_name(name);
        match path {
            Ok(p) => p.exists(),
            Err(_) => false,
        }
    }

    fn save_map(&self, name: &str, map: &MapState) -> std::io::Result<()> {
        let path = self.get_map_path_by_name(name)?;
        let json = map.as_json();
        let mut file = File::create(path)?;
        json.write_pretty(&mut file, 2)?;
        Ok(())
    }

    fn load_map<T: AsRef<Path>>(&self, name: T, room_state: &mut RoomState) -> std::io::Result<()> {
        let path = self.get_map_path_by_name(name)?;
        let json = JsonValue::from(fs::read_to_string(path)?);
        let mut msg = Message::new(MsgType::Map);
        msg.attach_body(MsgBody::Json(json));
        room_state.clear_map();
        room_state.send_msg(msg);
        Ok(())
    }

    fn get_map_path_by_name<T: AsRef<Path>>(&self, name: T) -> std::io::Result<PathBuf> {
        let mut fname = PathBuf::from(name.as_ref());
        fname.set_extension("json");
        Ok(self
            .map_dir
            .join(fname.file_name().ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid path",
            ))?))
    }
}
