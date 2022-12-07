use eframe::egui;
use egui::containers::panel::{CentralPanel, SidePanel};
use egui::containers::ScrollArea;

use egui::widget_text::RichText;
use egui::widgets::{Button, DragValue, ImageButton, Label};
use egui::{Align, Align2, Area, Color32, Context, Frame, Key, Layout, Ui};

use clipboard::{ClipboardContext, ClipboardProvider};

use std::collections::HashMap;
use std::path::PathBuf;

use crate::state::RoomState;
use crate::textures::Textures;
use crate::ui::widgets::RelArea;
use crate::ui::{MapUi, Window};
use crate::DraduError;

use crate::ui::window_tools::MapManager;

//const HEADER: &str = concat!("DRADU ", env!("CARGO_PKG_VERSION"));
const HEADER: &str = "DRADU ALPHA";

pub struct MainUi {
    map_ui: MapUi,
    textures: Textures,
    buffers: Buffers,
    current_tab: Tab,
    cwd: PathBuf,
    windowed_tools: HashMap<&'static str, WindowedTool>,
}

impl MainUi {
    pub fn new(textures: Textures) -> Self {
        let mut windowed_tools = HashMap::new();
        windowed_tools.insert(
            "Map manager",
            WindowedTool::new(Box::new(MapManager::default())),
        );
        MainUi {
            map_ui: MapUi::default(),
            textures,
            buffers: Buffers::default(),
            current_tab: Tab::Chat,
            cwd: PathBuf::from(""),
            windowed_tools,
        }
    }
}

impl MainUi {
    pub fn update(&mut self, ctx: &Context, room_state: &mut RoomState) -> Result<(), DraduError> {
        let screen_width = ctx.input().screen_rect().width();
        SidePanel::left("ul0")
            .min_width(self.buffers.tab_panel_width.unwrap_or(0.0) + 16.0)
            .max_width(screen_width / 2.0)
            .show(ctx, |ui| -> Result<(), DraduError> {
                ui.vertical_centered(|ui| {
                    ui.heading(HEADER);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    match self.buffers.tab_panel_width {
                        Some(w) => ui.add_space((ui.available_width() - w - 16.0) / 2.0),
                        None => (),
                    }
                    ui.group(|ui| {
                        if ui
                            .add(ImageButton::new(&self.textures["chat"], [22.0, 22.0]))
                            .clicked()
                        {
                            self.current_tab = Tab::Chat;
                        }
                        if ui
                            .add(ImageButton::new(&self.textures["info"], [22.0, 22.0]))
                            .clicked()
                        {
                            self.current_tab = Tab::Info;
                        }
                        if ui
                            .add(ImageButton::new(&self.textures["tools"], [22.0, 22.0]))
                            .clicked()
                        {
                            self.current_tab = Tab::Tools;
                        }
                        if room_state.is_master() {
                            if ui
                                .add(ImageButton::new(&self.textures["images"], [22.0, 22.0]))
                                .clicked()
                            {
                                self.current_tab = Tab::Images;
                            }
                        }
                        if ui
                            .add(ImageButton::new(&self.textures["settings"], [22.0, 22.0]))
                            .clicked()
                        {
                            self.current_tab = Tab::Settings;
                        }
                        self.buffers.tab_panel_width = Some(ui.min_rect().width());
                    });
                });

                ui.add_space(5.0);

                match self.current_tab {
                    Tab::Chat => self.display_chat(ui, room_state),
                    Tab::Info => self.display_info(ui, room_state)?,
                    Tab::Tools => self.display_tools(ui, room_state),
                    Tab::Images => self.display_images_dialog(ui, room_state),
                    Tab::Settings => self.display_settings(ui, room_state),
                };
                Ok(())
            })
            .inner?;

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::both()
                .auto_shrink([false, false])
                .always_show_scroll(true)
                .show(ui, |ui| {
                    self.map_ui.update(ui, room_state);
                })
        });
        self.display_map_overlay_ui(ctx);

        for tool in self.windowed_tools.values_mut() {
            if tool.open {
                tool.open = tool.win.show(ctx, room_state);
            }
        }

        Ok(())
    }

    fn display_map_overlay_ui(&mut self, ctx: &Context) {
        // UI to change scale of the map
        Area::new("ma0")
            .anchor(Align2::LEFT_TOP, (5.0, 5.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    Frame::popup(ui.style()).show(ui, |ui| {
                        if ui.button(RichText::new("+").monospace()).clicked() {
                            self.map_ui.global_scale += 0.1;
                        }
                        if ui.button(RichText::new("-").monospace()).clicked() {
                            self.map_ui.global_scale -= 0.1;
                        }
                        ui.label(format!("{:.0}%", self.map_ui.global_scale * 100.0));
                    });
                })
            });
        // User should also be able to change scale using Ctrl+Scrl
        self.map_ui.global_scale += ctx.input().zoom_delta() - 1.0;
        self.map_ui.global_scale = self.map_ui.global_scale.clamp(0.01, 10.0);
    }

    fn display_chat(&mut self, ui: &mut Ui, room_state: &mut RoomState) {
        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.add_space(5.0);
            let response = ui.add(Button::image_and_text(
                self.textures["send"].id(),
                [18.0, 18.0],
                "Send",
            ));
            if response.clicked() {
                room_state.send_chat_message(&self.buffers.chat_input);
                self.buffers.chat_input.clear();
            }
            response.on_hover_text("You can also use Ctrl+Enter");
            ui.add_space(5.0);
            if ui
                .text_edit_multiline(&mut self.buffers.chat_input)
                .has_focus()
            {
                if ui.input().modifiers.ctrl && ui.input().key_pressed(Key::Enter) {
                    room_state.send_chat_message(&self.buffers.chat_input);
                    self.buffers.chat_input.clear();
                }
            }
            ui.add_space(5.0);
            // FIXME: Text in log isn't selectable
            ui.vertical(|ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .always_show_scroll(true)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for chat_msg in room_state.chat_log_ref() {
                            let player = match room_state.get_player_by_id(&chat_msg.sender_id) {
                                Some(p) => p,
                                None => continue,
                            };
                            ui.add(repr_player(
                                player.1.clone(),
                                &player.0,
                                &chat_msg.sender_id,
                            ));
                            ui.label(&chat_msg.text);
                            ui.add_space(5.0);
                        }
                    });
            });
        });
    }

    fn display_info(&mut self, ui: &mut Ui, room_state: &RoomState) -> Result<(), DraduError> {
        ui.heading("Room address:");
        ui.horizontal(|ui| -> Result<(), DraduError> {
            let room_address = room_state.get_room_address()?;
            ui.label(&room_address);
            if ui.button("Copy").clicked() {
                let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
                clipboard.set_contents(room_address);
            }
            Ok(())
        })
        .inner?;
        ui.group(|ui| {
            for (id, (nickname, color)) in room_state.players_ref().iter() {
                ui.add(repr_player(color.clone(), nickname, id));
            }
        });
        Ok(())
    }

    fn display_tools(&mut self, ui: &mut Ui, room_state: &mut RoomState) {
        self.display_grid_settings(ui, room_state);
        ui.add_space(10.0);
        self.display_windowed_tools(ui, room_state);
    }

    fn display_windowed_tools(&mut self, ui: &mut Ui, _room_state: &mut RoomState) {
        ui.heading("Toolbox");
        ui.indent("ui1", |ui| {
            for (name, tool) in self.windowed_tools.iter_mut() {
                ui.horizontal(|ui| {
                    ui.label(*name);
                    ui.checkbox(&mut tool.open, "");
                });
            }
        });
    }

    fn display_grid_settings(&mut self, ui: &mut Ui, room_state: &mut RoomState) {
        ui.horizontal(|ui| {
            ui.heading("Grid");
            if ui.checkbox(&mut self.buffers.grid_enabled, "").changed() {
                if self.buffers.grid_enabled {
                    room_state.change_grid_size(self.buffers.grid_size);
                } else {
                    room_state.change_grid_size([0, 0]);
                }
            }
        });
        ui.add_enabled_ui(self.buffers.grid_enabled, |ui| {
            ui.indent("ui0", |ui| {
                ui.horizontal(|ui| {
                    let r1 =
                        ui.add(DragValue::new(&mut self.buffers.grid_size[0]).clamp_range(2..=255));
                    ui.label("by");
                    let r2 =
                        ui.add(DragValue::new(&mut self.buffers.grid_size[1]).clamp_range(2..=255));
                    let union = r1.union(r2);
                    if union.changed() {
                        room_state.change_grid_size(self.buffers.grid_size);
                    } else if !union.dragged() {
                        if let Some(grid_size) = room_state.map().grid {
                            self.buffers.grid_enabled = true;
                            self.buffers.grid_size = grid_size;
                        } else {
                            self.buffers.grid_enabled = false;
                        }
                    }
                });
            });
        });
    }

    fn display_images_dialog(&mut self, ui: &mut Ui, room_state: &mut RoomState) {
        if !ui.input().raw.hovered_files.is_empty() {
            RelArea::new("ur0")
                .align(Align2::CENTER_CENTER)
                .show_inside(ui, |ui| {
                    ui.heading("DROP FILE HERE");
                });
        }
        for file in ui.input().raw.dropped_files.iter() {
            room_state
                .fs_ref()
                .copy_into(file.path.as_ref().unwrap(), &self.cwd);
        }
        ScrollArea::horizontal().show(ui, |ui| {
            ui.label(format!("{}", self.cwd.display()));
            if let Some(_parent) = self.cwd.parent() {
                if ui.button("UP").clicked() {
                    self.cwd.pop();
                }
            }
            for entry in room_state.fs_ref().list_entries(&self.cwd).unwrap() {
                let entry = entry.unwrap();
                let filename_raw = entry.file_name();
                let filename = filename_raw.to_string_lossy();
                let is_dir = entry.file_type().unwrap().is_dir();
                let resp = ui.button(filename.as_ref());
                if resp.clicked() {
                    if is_dir {
                        self.cwd = self.cwd.join(filename.as_ref());
                    } else {
                        room_state.insert_decal(self.cwd.join(filename.as_ref()));
                    }
                }
                resp.context_menu(|ui| {
                    if ui.button("Set as BG image").clicked() {
                        room_state.set_background_image(self.cwd.join(filename.as_ref()));
                    }
                });
            }
            if let Some(ref mut string) = self.buffers.create_dir {
                let resp = ui.text_edit_singleline(string);
                if resp.lost_focus() {
                    if ui.input().key_pressed(Key::Enter) {
                        room_state.fs_ref().create_dir(&self.cwd.join(string));
                    }
                    self.buffers.create_dir = None;
                } else {
                    resp.request_focus();
                }
            } else {
                if ui.button("+").clicked() {
                    self.buffers.create_dir = Some(String::new());
                }
            }
        });
    }

    fn display_settings(&mut self, ui: &mut Ui, room_state: &mut RoomState) {
        ui.strong("Switch theme");
        // BUG(egui): this ratio does not work with vertical_centered Ui
        egui::widgets::global_dark_light_mode_buttons(ui);
        if ui.button("Quit").clicked() {
            room_state.quit_room();
        }
    }
}

// These are just data-structs used for organizing data which is persisted between frames
#[derive(PartialEq)]
enum Tab {
    Chat,
    Info,
    Tools,
    Images,
    Settings,
}

struct Buffers {
    grid_enabled: bool,
    grid_size: [u8; 2],
    tab_panel_width: Option<f32>,
    create_dir: Option<String>,
    chat_input: String,
}

impl Default for Buffers {
    fn default() -> Self {
        Self {
            grid_enabled: false,
            grid_size: [2, 2],
            tab_panel_width: None,
            create_dir: None,
            chat_input: String::new(),
        }
    }
}

struct WindowedTool {
    open: bool,
    win: Box<dyn Window>,
}

impl WindowedTool {
    fn new(win: Box<dyn Window>) -> Self {
        Self { open: false, win }
    }
}

fn repr_player(color: Color32, nick: &str, id: &str) -> Label {
    Label::new(
        RichText::new(format!("{}#{}", nick, &id[..4]))
            .color(color)
            .strong(),
    )
}
