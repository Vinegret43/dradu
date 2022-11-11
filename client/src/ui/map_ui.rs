use eframe::egui;
use egui::{Color32, Frame, Pos2, Stroke, Ui};

use crate::state::map::MapObject;
use crate::state::RoomState;
use crate::ui::widgets::{Dragging, RelArea};

pub struct MapUi {
    global_scale: f32, // TODO: Make this useful
    last_dragged_pos: Pos2,
    selected_item: Option<String>,
    selected_item_scale: f32,
}

impl Default for MapUi {
    fn default() -> Self {
        Self {
            global_scale: 1.0,
            last_dragged_pos: Pos2::new(0.0, 0.0),
            selected_item: None,
            selected_item_scale: 1.0,
        }
    }
}

// FIXME: When resizing object its size may become negative since there
// are no checks for that
impl MapUi {
    pub fn update(&mut self, ui: &mut Ui, room_state: &mut RoomState) {
        if let Some(ref image) = &room_state.map().background_image {
            room_state.get_image(image).show(ui);
        }

        let mut moved_object = None;
        let mut deleted_object = None;
        let mut rescaled_object = None;
        for (id, obj) in room_state.map().objects.iter() {
            match obj {
                MapObject::Decal(decal) => {
                    let image = room_state.get_image(&decal.path);
                    let (pos, resp) = RelArea::new(&decal.id)
                        .set_dragging(Dragging::Prioritized)
                        .set_pos(decal.pos)
                        .show_inside(ui, |ui| match &self.selected_item {
                            Some(selected_item) if selected_item == id => image.show_scaled(
                                ui,
                                decal.scale * self.global_scale * self.selected_item_scale,
                            ),
                            _ => image.show_scaled(ui, decal.scale * self.global_scale),
                        });
                    match &self.selected_item {
                        Some(selected_item) if selected_item == id => {
                            // Drawing frame around selected object.
                            // Using a tuple to make hash different from the one above
                            let (frame_pos, frame_resp) = RelArea::new((&decal.id, 0))
                                .set_dragging(Dragging::Disabled)
                                .ignore_bounds()
                                .set_pos(pos)
                                .show_inside(ui, |ui| {
                                    Frame::none().stroke(Stroke::new(2.0, Color32::GRAY)).show(
                                        ui,
                                        |ui| {
                                            ui.allocate_space(
                                                image.size_vec2()
                                                    * self.global_scale
                                                    * self.selected_item_scale
                                                    * decal.scale,
                                            );
                                        },
                                    );
                                    ui.horizontal(|ui| {
                                        if ui.button("Delete").clicked() {
                                            deleted_object = Some(id.clone());
                                        }
                                    });
                                });
                            // Drawing the resize thing (Slider)
                            let (slider_pos, slider_resp) = RelArea::new((&decal.id, 1))
                                .ignore_bounds()
                                .set_dragging(Dragging::Prioritized)
                                .set_pos(Pos2::new(
                                    (pos.x + image.width() as f32 / 2.0 * decal.scale) * self.global_scale,
                                    (pos.y + (image.height() as f32) * decal.scale) * self.global_scale,
                                ))
                                .show_inside(ui, |ui| {
                                    ui.label("R");
                                });
                            if slider_resp.response.drag_released() {
                                rescaled_object = Some((id.clone(), self.selected_item_scale * decal.scale));
                                self.selected_item_scale = 1.0;
                            }
                            self.selected_item_scale =
                                (slider_pos.y - pos.y) / (image.height() as f32 * decal.scale);
                            if resp.response.clicked_elsewhere() && !frame_resp.response.hovered() {
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
        if let Some((id, scale)) = rescaled_object {
            room_state.rescale_map_object(&id, scale);
        }
    }
}
