use eframe::egui;
use egui::{Color32, Frame, Image, Pos2, Rect, Response, Stroke, Ui, Vec2};

use crate::state::map::{MapObject, Token};
use crate::state::RoomState;
use crate::ui::widgets::{self, Dragging, RelArea, RelAreaResponse};

pub struct MapUi {
    pub global_scale: f32, // TODO: Make this useful
    last_dragged_pos: Pos2,
    selected_object: Option<String>,
    selected_object_scale: f32,
    snapping_enabled: bool,
    snap_to: Option<Pos2>, // Where to snap curently dragged item?
    map_size: Option<Vec2>,
}

impl Default for MapUi {
    fn default() -> Self {
        Self {
            global_scale: 1.0,
            last_dragged_pos: Pos2::new(0.0, 0.0),
            selected_object: None,
            selected_object_scale: 1.0,
            snapping_enabled: true,
            snap_to: None,
            map_size: None,
        }
    }
}

// FIXME: When resizing object its size may become negative since there
// are no checks for that
impl MapUi {
    pub fn update(&mut self, ui: &mut Ui, room_state: &mut RoomState) -> Response {
        Frame::none()
            .show(ui, |ui| {
                self.map_ui(ui, room_state);
            })
            .response
    }

    fn map_ui(&mut self, ui: &mut Ui, room_state: &mut RoomState) {
        let bg_resp = self.draw_bg_image(ui, room_state);
        self.map_size = bg_resp.and_then(|v| Some(v.rect.size()));
        if let Some(grid_size) = room_state.map().grid {
            widgets::draw_grid(grid_size, ui);
        }

        let mut map_action = MapAction::None;
        for (id, obj) in room_state.map().objects.iter() {
            let mut display_object = DisplayObject {
                id: &id,
                global_scale: self.global_scale,
                additional_scale_factor: 1.0,
                map_object: obj,
                room_state: room_state,
                frame_enabled: false,
            };
            let resp = match &self.selected_object {
                Some(sel_id) if id == sel_id => {
                    display_object.additional_scale_factor = self.selected_object_scale;
                    display_object.frame_enabled = true;
                    if let Some(snap_to) = self.snap_to {
                        display_object.place_as_snapping_guide(ui, snap_to)
                    }
                    let resp = display_object.place(ui);
                    // Additional UI
                    map_action = map_action.or(display_object.draw_ui(ui, &resp));
                    if resp.response.clicked_elsewhere() {
                        self.selected_object = None;
                    }
                    resp
                }
                _ => display_object.place(ui),
            };
            map_action = map_action.or(self.process_object_response(&display_object, resp));
        }
        map_action.apply(room_state);
    }

    fn process_object_response<T>(
        &mut self,
        obj: &DisplayObject,
        resp: RelAreaResponse<T>,
    ) -> MapAction {
        if resp.response.drag_started() || resp.response.clicked() {
            self.selected_object = Some(obj.id.to_owned());
        } else if resp.response.dragged() {
            self.last_dragged_pos = resp.current_pos;
            self.snap_to = None;
            if self.snapping_enabled {
                if let (Some(bg_size), Some(grid_size)) = (self.map_size, obj.room_state.map().grid)
                {
                    self.snap_to = Some(self.calculate_snap_pos(
                        grid_size,
                        bg_size,
                        resp.current_pos,
                        resp.response.rect.size(),
                    ));
                }
            };
        } else if resp.response.drag_released() {
            let pos = self.snap_to.take().unwrap_or(self.last_dragged_pos);
            return MapAction::Move(
                obj.id.to_owned(),
                (pos.to_vec2() / self.global_scale).to_pos2(),
            );
        }
        MapAction::None
    }

    fn draw_bg_image(&self, ui: &mut Ui, room_state: &RoomState) -> Option<Response> {
        match &room_state.map().background_image {
            Some(path) => Some(
                room_state
                    .get_image(&path)
                    .show_scaled(ui, self.global_scale),
            ),
            None => None,
        }
    }

    fn calculate_snap_pos(&self, grid_size: [u8; 2], bg_size: Vec2, pos: Pos2, size: Vec2) -> Pos2 {
        let image_center = pos + (size / 2.0);
        let column_width = bg_size.x / grid_size[0] as f32;
        let h_offset = image_center.x % column_width;
        let x = image_center.x - h_offset + column_width / 2.0;
        let row_height = bg_size.y / grid_size[1] as f32;
        let v_offset = image_center.y % row_height;
        let y = image_center.y - v_offset + row_height / 2.0;
        Pos2::new(x - size.x / 2.0, y - size.y / 2.0)
    }
}

enum MapAction {
    Move(String, Pos2),
    Delete(String),
    Rescale(String, f32),
    None,
}

impl MapAction {
    pub fn or(self, other: MapAction) -> MapAction {
        match self {
            MapAction::None => other,
            _ => self,
        }
    }
}

impl MapAction {
    fn apply(self, room_state: &mut RoomState) {
        match self {
            Self::Move(id, pos) => room_state.move_map_object(&id, pos),
            Self::Delete(id) => room_state.delete_map_object(&id),
            Self::Rescale(id, scale) => room_state.rescale_map_object(&id, scale),
            Self::None => (),
        };
    }
}

struct DisplayObject<'a> {
    pub id: &'a str,
    pub global_scale: f32,
    pub additional_scale_factor: f32,
    pub map_object: &'a MapObject,
    pub room_state: &'a RoomState,
    pub frame_enabled: bool,
}

impl<'a> DisplayObject<'a> {
    pub fn place(&self, ui: &mut Ui) -> RelAreaResponse<()> {
        let image = self.room_state.get_image(self.map_object.path());
        let resp = match self.map_object {
            MapObject::Decal(_) | MapObject::Token(_) => RelArea::new(self.id)
                .set_dragging(Dragging::Prioritized)
                .set_pos((self.map_object.pos().to_vec2() * self.global_scale).to_pos2())
                .show_inside(ui, |ui| {
                    image.show_scaled(
                        ui,
                        self.map_object.scale() * self.global_scale * self.additional_scale_factor,
                    );
                }),
            _ => unimplemented!(),
        };
        if self.frame_enabled {
            RelArea::new((self.id, 0))
                .set_dragging(Dragging::Disabled)
                .ignore_bounds()
                .set_pos(resp.current_pos)
                .show_inside(ui, |ui| {
                    Frame::none()
                        .stroke(Stroke::new(1.0, Color32::GRAY))
                        .show(ui, |ui| ui.allocate_space(resp.response.rect.size()))
                });
        }
        resp
    }

    // This will make the object non-interactive and slightly transparent
    pub fn place_as_snapping_guide(&self, ui: &mut Ui, pos: Pos2) {
        let image = self.room_state.get_image(self.map_object.path());
        let size = image.size_vec2()
            * self.map_object.scale()
            * self.global_scale
            * self.additional_scale_factor;
        let opaque_image = Image::new(image.texture_id(ui.ctx()), size)
            .tint(Color32::from_rgba_unmultiplied(255, 255, 255, 120));
        let pos = ui.min_rect().min + pos.to_vec2();
        opaque_image.paint_at(ui, Rect::from_min_max(pos, pos + size));
    }

    // Reponse from previously called .place
    pub fn draw_ui(&self, ui: &mut Ui, resp: &RelAreaResponse<()>) -> MapAction {
        match self.map_object {
            MapObject::Decal(_) => self.draw_decal_ui(ui, resp),
            MapObject::Token(token) => self.draw_token_ui(ui, resp, token),
            _ => unimplemented!(),
        }
    }

    pub fn draw_decal_ui(&self, ui: &mut Ui, resp: &RelAreaResponse<()>) -> MapAction {
        let mut action = MapAction::None;
        RelArea::new((self.id, 1))
            .set_dragging(Dragging::Disabled)
            .set_pos(resp.current_pos + Vec2::new(0.0, resp.response.rect.height()))
            .show_inside(ui, |ui| {
                if ui.button("Delete").clicked() {
                    action = MapAction::Delete(self.id.to_string());
                }
            });
        action
    }

    pub fn draw_token_ui(
        &self,
        ui: &mut Ui,
        resp: &RelAreaResponse<()>,
        token: &Token,
    ) -> MapAction {
        MapAction::None // TODO
    }
}
