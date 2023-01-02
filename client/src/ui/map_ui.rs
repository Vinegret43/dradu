use eframe::egui;
use egui::containers::Window;
use egui::{
    hex_color, Area, Color32, Frame, Image, Key, Pos2, Rect, Response, Rounding, Sense, Shape,
    Stroke, TextEdit, Ui, Vec2,
};

use egui::epaint::RectShape;

use std::cmp;

use crate::state::map::{MapObject, Token};
use crate::state::RoomState;
use crate::ui::widgets::{self, Dragging, RelArea, RelAreaResponse};
use crate::Textures;

pub struct MapUi {
    pub global_scale: f32,
    textures: Textures,
    last_dragged_pos: Pos2,
    selected_object: Option<String>,
    selected_object_scale: f32,
    snapping_enabled: bool,
    snap_to: Option<Pos2>, // Where to snap curently dragged item?
    map_size: Option<Vec2>,
    display_object_ui_state: DisplayObjectUiState,
}

impl MapUi {
    pub fn new(textures: Textures) -> Self {
        Self {
            global_scale: 1.0,
            textures,
            last_dragged_pos: Pos2::new(0.0, 0.0),
            selected_object: None,
            selected_object_scale: 1.0,
            snapping_enabled: true,
            snap_to: None,
            map_size: None,
            display_object_ui_state: DisplayObjectUiState::default(),
        }
    }
}

impl MapUi {
    pub fn update(&mut self, ui: &mut Ui, room_state: &mut RoomState) {
        ui.scope(|ui| {
            self.map_ui(ui, room_state);
            if ui
                .interact(ui.min_rect(), ui.id(), Sense::click())
                .clicked()
            {
                self.selected_object = None;
                self.selected_object_scale = 1.0;
            }
        });
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
                rescale_factor: 1.0,
                map_object: obj,
                room_state: room_state,
                is_selected: false,
            };
            let resp = match &self.selected_object {
                Some(sel_id) if id == sel_id => {
                    display_object.rescale_factor = self.selected_object_scale;
                    display_object.is_selected = true;
                    if let Some(snap_to) = self.snap_to {
                        display_object.place_as_snapping_guide(ui, snap_to)
                    }
                    let resp = display_object.place(ui);
                    // Additional UI
                    map_action = map_action.or(display_object.draw_ui(
                        ui,
                        &resp,
                        &mut self.display_object_ui_state,
                    ));
                    map_action = map_action.or(self.draw_resize_slider(&display_object, ui, &resp));
                    resp
                }
                _ => display_object.place(ui),
            };
            map_action = map_action.or(self.process_object_response(&display_object, resp));
        }
        map_action.apply(room_state);
    }

    fn draw_resize_slider(
        &mut self,
        obj: &DisplayObject,
        ui: &mut Ui,
        resp: &RelAreaResponse<()>,
    ) -> MapAction {
        let slider_resp = Area::new("resize")
            .current_pos(resp.response.rect.max - Vec2::new(16.0, 16.0))
            .show(ui.ctx(), |ui| {
                ui.image(&self.textures["resize"], [16.0, 16.0]);
            })
            .response;
        let curr_rect_size = resp.response.rect.size();
        let new_rect_size = curr_rect_size + slider_resp.drag_delta();
        if new_rect_size.min_elem() >= 24.0 {
            self.selected_object_scale *= (new_rect_size / curr_rect_size).min_elem();
        }
        if slider_resp.drag_released() {
            let action = MapAction::Rescale(
                obj.id.to_string(),
                obj.map_object.scale() * self.selected_object_scale,
            );
            self.selected_object_scale = 1.0;
            action
        } else {
            MapAction::None
        }
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
    UpdateTokenProperty(String, String, String),
    RemoveTokenProperty(String, String),
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
            Self::UpdateTokenProperty(id, k, v) => room_state.update_token_property(&id, &k, &v),
            Self::RemoveTokenProperty(id, k) => room_state.remove_token_property(&id, &k),
            Self::None => (),
        };
    }
}

struct DisplayObject<'a> {
    pub id: &'a str,
    pub global_scale: f32,
    pub rescale_factor: f32,
    pub map_object: &'a MapObject,
    pub room_state: &'a RoomState,
    pub is_selected: bool,
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
                        self.map_object.scale() * self.global_scale * self.rescale_factor,
                    );
                }),
            _ => unimplemented!(),
        };
        if self.is_selected {
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
        let size =
            image.size_vec2() * self.map_object.scale() * self.global_scale * self.rescale_factor;
        let opaque_image = Image::new(image.texture_id(ui.ctx()), size)
            .tint(Color32::from_rgba_unmultiplied(255, 255, 255, 120));
        let pos = ui.min_rect().min + pos.to_vec2();
        opaque_image.paint_at(ui, Rect::from_min_max(pos, pos + size));
    }

    // Reponse from previously called .place
    pub fn draw_ui(
        &self,
        ui: &mut Ui,
        resp: &RelAreaResponse<()>,
        ui_state: &mut DisplayObjectUiState,
    ) -> MapAction {
        match self.map_object {
            MapObject::Decal(_) => self.draw_delete_button(ui, resp),
            MapObject::Token(token) => self.draw_token_ui(ui, resp, token, ui_state),
            _ => unimplemented!(),
        }
    }

    pub fn draw_delete_button(&self, ui: &mut Ui, resp: &RelAreaResponse<()>) -> MapAction {
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
        ui_state: &mut DisplayObjectUiState,
    ) -> MapAction {
        let mut map_action = MapAction::None;
        map_action = map_action.or(self.draw_delete_button(ui, resp));

        if self.id != ui_state.last_id {
            ui_state.last_id = self.id.to_string();
            ui_state.edited_key = None;
            ui_state.edited_value = None;
        }

        Window::new("Token properties")
            .show(ui.ctx(), |ui| {
                if let Some(ref mut key) = ui_state.edited_key {
                    if let Some(ref mut val) = ui_state.edited_value {
                        // Editing value
                        for (k, v) in token.properties.iter() {
                            if k == key {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}:", k));
                                    let resp = ui.add(narrow_text_edit(val));
                                    if ui.button("✔").clicked()
                                        || resp.lost_focus() && ui.input().key_pressed(Key::Enter)
                                    {
                                        map_action = MapAction::UpdateTokenProperty(
                                            self.id.to_string(),
                                            k.to_string(),
                                            val.to_string(),
                                        );
                                    }
                                });
                            } else {
                                ui.label(format!("{}: {}", k, v));
                            }
                        }
                        if ui.button("Add").clicked() {
                            ui_state.edited_key = Some(String::new());
                            ui_state.edited_value = None;
                        }
                    } else {
                        // Adding value
                        for (k, v) in token.properties.iter() {
                            ui.label(format!("{}: {}", k, v));
                        }
                        ui.horizontal(|ui| {
                            let resp = ui.add(narrow_text_edit(key));
                            if ui.button("✔").clicked()
                                || resp.lost_focus() && ui.input().key_pressed(Key::Enter)
                            {
                                let (key, val) = key.split_once(':').unwrap_or((key, ""));
                                map_action = MapAction::UpdateTokenProperty(
                                    self.id.to_string(),
                                    key.to_string(),
                                    val.to_string(),
                                );
                            }
                        });
                    }
                } else {
                    // Normal display
                    for (k, v) in token.properties.iter() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}: {}", k, v));
                            if ui.button("✏").clicked() {
                                ui_state.edited_key = Some(k.to_string());
                                ui_state.edited_value = Some(v.to_string());
                            }
                            if ui.button("X").clicked() {
                                map_action = MapAction::RemoveTokenProperty(
                                    self.id.to_string(),
                                    k.to_string(),
                                );
                            }
                        });
                    }
                    if ui.button("Add").clicked() {
                        ui_state.edited_key = Some(String::new());
                    }
                }
            });

        map_action = map_action.or(self.draw_token_bars(ui, resp, token));

        if let MapAction::UpdateTokenProperty(_, _, _) = map_action {
            ui_state.edited_key = None;
            ui_state.edited_value = None;
        }

        map_action
    }

    pub fn draw_token_bars(
        &self,
        ui: &mut Ui,
        resp: &RelAreaResponse<()>,
        token: &Token,
    ) -> MapAction {
        let width = resp.response.rect.width();
        let mut pos = resp.response.rect.center_bottom() + Vec2::new(0.0, 30.0 * self.global_scale);
        if let Some(bar) = token.properties.get("red_bar") {
            let (fullness, s) = match token.properties.get("red_bar_max") {
                Some(max) => match (bar.parse::<f32>(), max.parse::<f32>()) {
                    (Ok(bar), Ok(max)) => ((bar / max), format!("{}/{}", bar, max)),
                    _ => (0.0, bar.to_string()),
                },
                None => (0.0, bar.to_string()),
            };
            self.draw_bar(ui, pos, &s, hex_color!("#ff5555aa"), fullness, width);
            pos += Vec2::new(0.0, 25.0 * self.global_scale);
        }
        if let Some(bar) = token.properties.get("blue_bar") {
            let (fullness, s) = match token.properties.get("blue_bar_max") {
                Some(max) => match (bar.parse::<f32>(), max.parse::<f32>()) {
                    (Ok(bar), Ok(max)) => ((bar / max), format!("{}/{}", bar, max)),
                    _ => (0.0, bar.to_string()),
                },
                None => (0.0, bar.to_string()),
            };
            self.draw_bar(ui, pos, &s, hex_color!("#7777ffaa"), fullness, width);
            pos += Vec2::new(0.0, 25.0 * self.global_scale);
        }
        if let Some(bar) = token.properties.get("green_bar") {
            let (fullness, s) = match token.properties.get("green_bar_max") {
                Some(max) => match (bar.parse::<f32>(), max.parse::<f32>()) {
                    (Ok(bar), Ok(max)) => ((bar / max), format!("{}/{}", bar, max)),
                    _ => (0.0, bar.to_string()),
                },
                None => (0.0, bar.to_string()),
            };
            self.draw_bar(ui, pos, &s, hex_color!("#22cc22aa"), fullness, width);
        }
        MapAction::None
    }

    fn draw_bar(
        &self,
        ui: &mut Ui,
        center_pos: Pos2,
        s: &str,
        color: Color32,
        fullness: f32,
        width: f32,
    ) {
        let rect = Rect::from_center_size(center_pos, Vec2::new(width, 20.0));
        let mut shapes = vec![rect_shape(
            rect,
            color,
            (2.0, change_color_lightness(color, -10)),
        )];
        let mut rect2 = rect.clone();
        rect2.set_width(rect.width() * fullness);
        rect2 = rect2.shrink(2.0);
        shapes.push(rect_shape(
            rect2,
            change_color_lightness(color, -20),
            (0.0, Color32::TRANSPARENT),
        ));
        let painter = ui.painter();
        painter.extend(shapes);
        ui.allocate_ui_at_rect(rect, |ui| {
            ui.horizontal_centered(|ui| {
                ui.colored_label(Color32::WHITE, s);
            });
        });
    }
}

// Pass this to every DisplayObject when drawing UI. This struct should be persisted between frames
#[derive(Default)]
struct DisplayObjectUiState {
    last_id: String,
    edited_key: Option<String>,
    edited_value: Option<String>,
}

fn narrow_text_edit(buf: &mut String) -> TextEdit {
    TextEdit::singleline(buf).desired_width(120.0)
}

fn rect_shape(rect: Rect, fill: impl Into<Color32>, stroke: impl Into<Stroke>) -> Shape {
    Shape::Rect(RectShape {
        rect,
        rounding: Rounding::from(0.0),
        fill: fill.into(),
        stroke: stroke.into(),
    })
}

fn change_color_lightness(color: Color32, delta: i8) -> Color32 {
    let rgba = color.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied(
        cmp::max(rgba[0] as i8 + delta, 0) as u8,
        cmp::max(rgba[1] as i8 + delta, 0) as u8,
        cmp::max(rgba[2] as i8 + delta, 0) as u8,
        rgba[3],
    )
}
