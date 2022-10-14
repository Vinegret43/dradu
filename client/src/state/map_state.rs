// TODO: Setting background image. Move to IndexedMap from HashMap to implement layering

use eframe::egui;
use egui::Pos2;
use egui_extras::RetainedImage;

use json::JsonValue;

use std::collections::HashMap;

pub struct MapState {
    pub objects: HashMap<String, MapObject>,
    pub background_image: Option<RetainedImage>,
    // Is guaranteed to be at least 2x2 or None
    grid: Option<(u16, u16)>,
}

impl MapState {
    pub fn grid(&self) -> Option<(u16, u16)> {
        self.grid
    }

    pub fn set_grid(&mut self, grid: Option<(u16, u16)>) {
        self.grid = match grid {
            Some((x, y)) if x > 1 && y > 1 => Some((x, y)),
            _ => None,
        }
    }
}

impl Default for MapState {
    fn default() -> Self {
        MapState {
            objects: HashMap::new(),
            background_image: None,
            grid: None,
        }
    }
}

pub enum MapObject {
    Decal(Decal),
    Token(Token),
    Wall(Wall),
}

pub struct Decal {
    pub id: String,
    pub pos: Pos2,
    pub scale: f32,
    pub path: String,
}

pub struct Token {
    pub id: String,
    pub pos: Pos2,
    pub scale: f32,
    pub path: String,
    // Additional things like health, armor, etc.
    pub attributes: JsonValue,
}

pub struct Wall {
    id: String,
    nodes: Vec<Pos2>,
    path: String,
}
