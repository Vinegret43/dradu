use eframe::egui;
use egui::widgets::Image;
use egui::{Color32, Frame, Pos2, Rect, Stroke, Ui};

use crate::state::map::MapObject;
use crate::state::RoomState;
use crate::ui::widgets::{draw_grid, Dragging, RelArea};

pub struct MapUi {
    global_scale: f32, // TODO: Make this useful
    last_dragged_pos: Pos2,
    selected_item: Option<String>,
    selected_item_scale: f32,
    snapping_enabled: bool,
    snap_to: Option<Pos2>, // Where to snap curently dragged item?
}

impl Default for MapUi {
    fn default() -> Self {
        Self {
            global_scale: 1.0,
            last_dragged_pos: Pos2::new(0.0, 0.0),
            selected_item: None,
            selected_item_scale: 1.0,
            snapping_enabled: true,
            snap_to: None,
        }
    }
}

// FIXME: When resizing object its size may become negative since there
// are no checks for that
impl MapUi {
    pub fn update(&mut self, ui: &mut Ui, room_state: &mut RoomState) {
        let bg_image_size = match &room_state.map().background_image {
            Some(ref path) => {
                let image = room_state.get_image(path);
                image.show(ui);
                Some(image.size_vec2())
            }
            None => None,
        };

        let mut moved_object = None;
        let mut deleted_object = None;
        let mut rescaled_object = None;
        for (id, obj) in room_state.map().objects.iter() {
            match obj {
                MapObject::Decal(decal) => {
                    let image = room_state.get_image(&decal.path);

                    // Showing where currently dragged object will be snapped to
                    if let Some(snap_to) = self.snap_to {
                        match &self.selected_item {
                            Some(selected_item) if selected_item == id => {
                                let size = image.size_vec2()
                                    * decal.scale
                                    * self.global_scale
                                    * self.selected_item_scale;
                                let opaque_image = Image::new(image.texture_id(ui.ctx()), size)
                                    .tint(Color32::from_rgba_unmultiplied(255, 255, 255, 120));
                                let pos = ui.min_rect().min + snap_to.to_vec2();
                                opaque_image.paint_at(ui, Rect::from_min_max(pos, pos + size));
                            }
                            _ => (),
                        }
                    }

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
                            let (_, frame_resp) = RelArea::new((&decal.id, 0))
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
                                    (pos.x + image.width() as f32 / 2.0 * decal.scale)
                                        * self.global_scale,
                                    (pos.y + (image.height() as f32) * decal.scale)
                                        * self.global_scale,
                                ))
                                .show_inside(ui, |ui| {
                                    ui.strong("R");
                                });
                            if slider_resp.response.drag_released() {
                                rescaled_object =
                                    Some((id.clone(), self.selected_item_scale * decal.scale));
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
                        // Snaps to the center of grid, not the side
                        self.snap_to = None;
                        if self.snapping_enabled {
                            if let (Some(bg_size), Some(grid_size)) =
                                (bg_image_size, room_state.map().grid)
                            {
                                let size = image.size_vec2()
                                    * decal.scale
                                    * self.global_scale
                                    * self.selected_item_scale;
                                let image_center = pos + (size / 2.0);
                                let column_width = bg_size.x / grid_size[0] as f32;
                                let h_offset = image_center.x % column_width;
                                let x = image_center.x - h_offset + column_width / 2.0;
                                let row_height = bg_size.y / grid_size[1] as f32;
                                let v_offset = image_center.y % row_height;
                                let y = image_center.y - v_offset + row_height / 2.0;
                                self.snap_to = Some(Pos2::new(x - size.x / 2.0, y - size.y / 2.0));
                            }
                        }
                    } else if resp.response.drag_released() {
                        moved_object = match self.snap_to.take() {
                            Some(snap_to) => Some((id.clone(), snap_to)),
                            None => Some((id.clone(), self.last_dragged_pos)),
                        };
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

        if let Some(grid_size) = room_state.map().grid {
            draw_grid(grid_size, ui);
        }
    }
}
