// TODO: Setting background image. Move to IndexedMap from HashMap to implement layering

use eframe::egui;
use egui::Pos2;

use json::JsonValue;

use indexmap::IndexMap;

pub struct MapState {
    pub objects: IndexMap<String, MapObject>,
    pub background_image: Option<String>,
    // Columns, rows
    pub grid: Option<[u8; 2]>,
}

impl Default for MapState {
    fn default() -> Self {
        MapState {
            objects: IndexMap::new(),
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
