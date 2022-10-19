use eframe::egui;
use egui::{Color32, Frame, Pos2, Stroke, Ui};

use crate::state::map::MapObject;
use crate::state::RoomState;
use crate::ui::widgets::{Dragging, RelArea};

pub struct MapUi {
    global_scale: f32, // TODO: Make this useful
    last_dragged_pos: Pos2,
    selected_item: Option<String>,
}

impl Default for MapUi {
    fn default() -> Self {
        Self {
            global_scale: 1.0,
            last_dragged_pos: Pos2::new(0.0, 0.0),
            selected_item: None,
        }
    }
}

impl MapUi {
    pub fn update(&mut self, ui: &mut Ui, room_state: &mut RoomState) {
        if let Some(ref image) = &room_state.map().background_image {
            room_state.get_image(image).show(ui);
        }

        let mut moved_object = None;
        let mut deleted_object = None;
        for (id, obj) in room_state.map().objects.iter() {
            match obj {
                MapObject::Decal(decal) => {
                    let image = room_state.get_image(&decal.path);
                    let (pos, resp) = RelArea::new(&decal.id)
                        .set_dragging(Dragging::Prioritized)
                        .set_pos(decal.pos)
                        .show_inside(ui, |ui| {
                            image.show_scaled(ui, decal.scale * self.global_scale);
                        });
                    match &self.selected_item {
                        Some(selected_item) if selected_item == id => {
                            // Using a tuple to make hash different from the one above
                            let (_, frame_resp) = RelArea::new((&decal.id, 0))
                                .set_dragging(Dragging::Disabled)
                                .ignore_bounds()
                                .set_pos(pos)
                                .show_inside(ui, |ui| {
                                    Frame::none().stroke(Stroke::new(2.0, Color32::GRAY)).show(
                                        ui,
                                        |ui| {
                                            ui.allocate_space(image.size_vec2() * self.global_scale)
                                        },
                                    );
                                    if ui.button("Delete").clicked() {
                                        deleted_object = Some(id.clone());
                                    }
                                });
                            if resp.response.clicked_elsewhere() {
                                self.selected_item = None;
                            }
                        }
                        _ => (),
                    }
                    if resp.response.drag_started() || resp.response.clicked() {
                        self.selected_item = Some(id.to_string());
                    } else if resp.response.dragged() {
                        self.last_dragged_pos = pos;
                    } else if resp.response.drag_released() {
                        moved_object = Some((id.clone(), self.last_dragged_pos));
                    }
                }
                MapObject::Token(token) => {
                    RelArea::new(&token.id)
                        .set_dragging(Dragging::Prioritized)
                        .set_pos(token.pos)
                        .show_inside(ui, |ui| {
                            room_state
                                .get_image(&token.path)
                                .show_scaled(ui, token.scale);
                        });
                }
                _ => (),
            }
        }
        if let Some((id, pos)) = moved_object {
            room_state.move_map_object(&id, pos);
        }
        if let Some(id) = deleted_object {
            room_state.delete_map_object(&id);
        }
    }
}
