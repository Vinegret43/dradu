// TODO: Setting background image. Move to IndexedMap from HashMap to implement layering

use eframe::egui;
use egui::Pos2;

use json::JsonValue;

use indexmap::IndexMap;

use crate::utils;
use crate::DraduError;

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

impl MapObject {
    pub fn update_from_json(&mut self, json: &JsonValue) -> Result<(), DraduError> {
        match self {
            MapObject::Decal(decal) => decal.update_from_json(json),
            MapObject::Token(token) => token.update_from_json(json),
            MapObject::Wall(_) => unimplemented!(),
        }
    }

    pub fn create_from_json(json: &JsonValue) -> Result<Self, DraduError> {
        match json["type"].as_str().ok_or(DraduError::ProtocolError)? {
            "token" => Ok(Self::Token(Token::create_from_json(json)?)),
            "decal" => Ok(Self::Decal(Decal::create_from_json(json)?)),
            "wall" => unimplemented!(),
            "effect" => unimplemented!(),
            _ => Err(DraduError::ProtocolError),
        }
    }

    pub fn pos(&self) -> Pos2 {
        match self {
            Self::Decal(decal) => decal.pos,
            Self::Token(token) => token.pos,
            Self::Wall(wall) => wall.pos,
        }
    }

    pub fn scale(&self) -> f32 {
        match self {
            Self::Decal(decal) => decal.scale,
            Self::Token(token) => token.scale,
            Self::Wall(_) => 1.0,
        }
    }

    pub fn path(&self) -> &str {
        match self {
            Self::Decal(decal) => &decal.path,
            Self::Token(token) => &token.path,
            Self::Wall(wall) => &wall.path,
        }
    }
}

pub struct Decal {
    pub pos: Pos2,
    pub scale: f32,
    pub path: String,
}

impl Decal {
    fn update_from_json(&mut self, json: &JsonValue) -> Result<(), DraduError> {
        if let Ok(pos) = utils::json_to_pos(&json["pos"]) {
            self.pos = pos;
        }
        if let Some(scale) = json["scale"].as_f32() {
            self.scale = scale;
        }
        Ok(())
    }

    fn create_from_json(json: &JsonValue) -> Result<Self, DraduError> {
        Ok(Self {
            pos: utils::json_to_pos(&json["pos"]).unwrap_or(Pos2::new(0.0, 0.0)),
            scale: json["scale"].as_f32().unwrap_or(1.0),
            path: json["path"]
                .as_str()
                .ok_or(DraduError::ProtocolError)?
                .to_owned(),
        })
    }
}

pub struct Token {
    pub pos: Pos2,
    pub scale: f32,
    pub path: String,
    // Additional things like health, armor, etc.
    pub attributes: JsonValue,
}

impl Token {
    fn update_from_json(&mut self, json: &JsonValue) -> Result<(), DraduError> {
        if let Ok(pos) = utils::json_to_pos(&json["pos"]) {
            self.pos = pos;
        }
        if let Some(scale) = json["scale"].as_f32() {
            self.scale = scale;
        }
        Ok(())
    }

    fn create_from_json(json: &JsonValue) -> Result<Self, DraduError> {
        Ok(Self {
            pos: utils::json_to_pos(&json["pos"]).unwrap_or(Pos2::new(0.0, 0.0)),
            scale: json["scale"].as_f32().unwrap_or(1.0),
            path: json["path"]
                .as_str()
                .ok_or(DraduError::ProtocolError)?
                .to_owned(),
            attributes: json["attributes"].clone(),
        })
    }
}

pub struct Wall {
    pub pos: Pos2,
    pub nodes: Vec<Pos2>,
    pub path: String,
}
