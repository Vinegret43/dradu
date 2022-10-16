use eframe::egui;
use egui::{Pos2, Sense, Ui, Vec2};

use crate::state::map::MapObject;
use crate::state::RoomState;
use crate::ui::widgets::{Dragging, RelArea};

#[derive(Default)]
pub struct MapUi {
    last_dragged_pos: Pos2,
}

impl MapUi {
    pub fn update(&mut self, ui: &mut Ui, room_state: &mut RoomState) {
        if let Some(ref image) = &room_state.map().background_image {
            room_state.get_image(image).show(ui);
        } else {
            ui.allocate_exact_size(Vec2::from([500.0, 500.0]), Sense::click());
        }

        let mut moved_object = None;
        for (id, obj) in room_state.map().objects.iter() {
            match obj {
                MapObject::Decal(decal) => {
                    let (pos, resp) = RelArea::new(&decal.id)
                        .set_dragging(Dragging::Prioritized)
                        .set_pos(decal.pos)
                        .show_inside(ui, |ui| {
                            room_state
                                .get_image(&decal.path)
                                .show_scaled(ui, decal.scale);
                        });
                    if resp.response.dragged() {
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
    }
}
