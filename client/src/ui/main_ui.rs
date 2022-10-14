use eframe::egui;
use egui::containers::panel::{CentralPanel, SidePanel};
use egui::containers::ScrollArea;

use egui::widget_text::RichText;
use egui::widgets::{Button, ImageButton, Label};
use egui::{Align, Align2, Color32, Context, Key, Layout, Ui};

use clipboard::{ClipboardContext, ClipboardProvider};

use std::path::PathBuf;

use crate::state::RoomState;
use crate::textures::Textures;
use crate::ui::{MapUi, RelArea};
use crate::DraduError;

//const HEADER: &str = concat!("DRADU ", env!("CARGO_PKG_VERSION"));
const HEADER: &str = "DRADU ALPHA";

pub struct MainUi {
    map_ui: MapUi,
    textures: Textures,
    geometries: Geometries,
    text_buffers: TextBuffers,
    current_tab: Tab,
    cwd: PathBuf,
}

impl MainUi {
    pub fn new(textures: Textures) -> Self {
        MainUi {
            map_ui: MapUi::default(),
            textures,
            geometries: Geometries::default(),
            text_buffers: TextBuffers::default(),
            current_tab: Tab::Chat,
            cwd: PathBuf::from(""),
        }
    }
}

impl MainUi {
    pub fn update(&mut self, ctx: &Context, room_state: &mut RoomState) -> Result<(), DraduError> {
        let screen_width = ctx.input().screen_rect().width();
        SidePanel::left("ul0")
            .min_width(self.geometries.tab_panel_width.unwrap_or(0.0) + 16.0)
            .max_width(screen_width / 2.0)
            .show(ctx, |ui| -> Result<(), DraduError> {
                ui.vertical_centered(|ui| {
                    ui.heading(HEADER);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    match self.geometries.tab_panel_width {
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
                        self.geometries.tab_panel_width = Some(ui.min_rect().width());
                    });
                });

                ui.add_space(5.0);

                match self.current_tab {
                    Tab::Chat => self.display_chat(ui, room_state),
                    Tab::Info => self.display_info(ui, room_state)?,
                    Tab::Tools => self.display_tools(ui),
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
                });
        });
        Ok(())
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
                room_state.send_chat_message(&self.text_buffers.chat_input);
                self.text_buffers.chat_input.clear();
            }
            response.on_hover_text("You can also use Ctrl+Enter");
            ui.add_space(5.0);
            if ui
                .text_edit_multiline(&mut self.text_buffers.chat_input)
                .has_focus()
            {
                if ui.input().modifiers.ctrl && ui.input().key_pressed(Key::Enter) {
                    room_state.send_chat_message(&self.text_buffers.chat_input);
                    self.text_buffers.chat_input.clear();
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

    fn display_tools(&mut self, ui: &mut Ui) {
        ui.label("TOOLS");
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
            if let Some(ref mut string) = self.text_buffers.create_dir {
                let resp = ui.text_edit_singleline(string);
                if resp.lost_focus() {
                    if ui.input().key_pressed(Key::Enter) {
                        room_state.fs_ref().create_dir(&self.cwd.join(string));
                    }
                    self.text_buffers.create_dir = None;
                } else {
                    resp.request_focus();
                }
            } else {
                if ui.button("+").clicked() {
                    self.text_buffers.create_dir = Some(String::new());
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

#[derive(Default)]
struct Geometries {
    tab_panel_width: Option<f32>,
}

#[derive(Default)]
struct TextBuffers {
    create_dir: Option<String>,
    chat_input: String,
}

fn repr_player(color: Color32, nick: &str, id: &str) -> Label {
    Label::new(
        RichText::new(format!("{}#{}", nick, &id[..4]))
            .color(color)
            .strong(),
    )
}