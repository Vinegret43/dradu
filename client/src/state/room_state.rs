// TODO: This file has some very bloated and ugly methods, they need refactoring

use eframe::egui;
use egui::{Color32, Context, Pos2};
use egui_extras::RetainedImage;

use json::{array, JsonValue};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;

use crate::fs::AssetDirHandler;
use crate::net::{Connection, Message, MsgBody, MsgType};
use crate::state::map::{MapState, Decal, MapObject, Token};
use crate::utils;
use crate::DraduError;

// This struct monitors and provides access to things like chat log,
// map, images, list of players and permissions. It also manages the
// server connection, updating all of these things when new messages
// come in. Also used for sending messages

pub struct RoomState {
    // permissions
    chat_log: Vec<ChatMessage>,
    master: bool,
    connection: Connection,
    fs: AssetDirHandler,

    players: HashMap<String, (String, Color32)>, // Id: (Nickname, Color)
    // It promises that all images referenced in map *will* be here,
    // however, there is a placeholder image which will be returned otherwise
    images: HashMap<String, RetainedImage>,
    map: MapState,
}

impl<'a> RoomState {
    pub fn join_room(addr: SocketAddr, room_id: &str, ctx: &Context) -> Result<Self, DraduError> {
        let connection = Connection::join_room(addr, room_id, ctx)?;
        Ok(Self::with_connection(connection, false))
    }

    pub fn create_new_room(addr: SocketAddr, ctx: &Context) -> Result<Self, DraduError> {
        let connection = Connection::create_new_room(addr, ctx)?;
        Ok(Self::with_connection(connection, true))
    }

    fn with_connection(connection: Connection, master: bool) -> Self {
        let mut images = HashMap::new();
        images.insert("placeholder".to_string(), utils::get_placeholder_image());

        let mut players = HashMap::new();
        players.insert(
            connection.get_user_id().to_string(),
            (
                connection.get_nickname().to_string(),
                connection.get_user_color(),
            ),
        );

        RoomState {
            chat_log: Vec::new(),
            master,
            connection,
            fs: AssetDirHandler::new(),
            players,
            images,
            map: MapState::default(),
        }
    }

    pub fn reconnect(&mut self) {
        todo!();
    }

    // Call this every frame to keep states up to date
    pub fn update_self(&mut self) -> Result<(), DraduError> {
        let new_messages = self.connection.new_messages()?;
        for mut msg in new_messages {
            match msg.msg_type() {
                MsgType::Map => {
                    if let Some(MsgBody::Json(json)) = msg.take_body() {
                        self.update_map(json)?;
                    }
                }
                MsgType::Player => {
                    if let Some(MsgBody::Json(json)) = msg.take_body() {
                        self.update_players(json)?;
                    }
                }
                MsgType::Msg => {
                    if let Some(MsgBody::Text(text)) = msg.take_body() {
                        if let Some(user_id) = msg.get_prop("userId") {
                            self.update_chat_log(user_id.to_owned(), text);
                        }
                    }
                }
                MsgType::Perm => {
                    if let Some(MsgBody::Json(json)) = msg.take_body() {
                        self.update_permissions(json); // TODO
                    }
                }
                MsgType::File => {
                    if self.master {
                        if let Some(s) = msg.get_prop("path") {
                            if let Ok(bytes) = self.fs.read_file(s) {
                                let mut response = Message::new(MsgType::File)
                                    .set_prop("path", s)
                                    .set_prop("contentType", "image");
                                response.attach_body(MsgBody::Bin(bytes));
                                self.send_msg(response)?;
                            }
                        }
                    } else {
                        if let Some(MsgBody::Image(image)) = msg.take_body() {
                            self.add_image(msg.get_prop("path").unwrap_or(""), image)
                        }
                    }
                }
                MsgType::Synced => {}
                MsgType::Err => (),
                _ => (),
            }
        }
        Ok(())
    }

    pub fn fs_ref(&self) -> &AssetDirHandler {
        &self.fs
    }

    pub fn get_room_address(&self) -> Result<String, DraduError> {
        self.connection.get_room_address()
    }

    pub fn send_msg(&mut self, message: Message) -> Result<usize, DraduError> {
        self.connection.send_msg(message)
    }

    pub fn request_file(&mut self, path: &str) {
        let msg = Message::new(MsgType::File).set_prop("path", path);
        self.send_msg(msg);
    }

    pub fn quit_room(&mut self) {
        self.send_msg(Message::new(MsgType::Quit));
        self.connection.close()
    }

    pub fn send_chat_message(&mut self, text: &str) {
        let mut msg = Message::new(MsgType::Msg);
        msg.attach_body(MsgBody::Text(text.to_owned()));
        self.send_msg(msg);
        if !text.starts_with('/') {
            self.update_chat_log(self.connection.get_user_id().to_owned(), text.to_owned());
        }
    }

    pub fn insert_decal<P: AsRef<Path>>(&mut self, path: P) -> Result<(), DraduError> {
        let path = path.as_ref();
        let path_str = path.to_str().unwrap();
        if !self.images.contains_key(path_str) {
            let image = self.fs.get_retained_image(path)?;
            self.images.insert(String::from(path_str), image);
        }
        let mut msg = Message::new(MsgType::Map);
        let mut inner_json = JsonValue::new_object();
        inner_json.insert("type", "decal");
        inner_json.insert("path", path.to_str());

        let mut json = JsonValue::new_object();
        json[utils::random_id()] = inner_json;
        msg.attach_body(MsgBody::Json(json));
        self.send_msg(msg);

        Ok(())
    }

    // FIXME: No bounds, nor permission checks
    pub fn move_map_object(&mut self, id: &str, pos: Pos2) {
        if self.map.objects.contains_key(id) {
            let mut msg = Message::new(MsgType::Map);
            let mut inner_json = JsonValue::new_object();
            inner_json.insert("pos", array![pos.x, pos.y]);
            let mut json = JsonValue::new_object();
            json.insert(id, inner_json);
            msg.attach_body(MsgBody::Json(json));
            self.send_msg(msg);
        }
    }

    pub fn chat_log_ref(&self) -> &Vec<ChatMessage> {
        &self.chat_log
    }

    pub fn players_ref(&self) -> &HashMap<String, (String, Color32)> {
        &self.players
    }

    pub fn get_user_id(&self) -> &str {
        self.connection.get_user_id()
    }

    pub fn get_nickname(&self) -> &str {
        &self.players[self.get_user_id()].0
    }

    pub fn get_user_color(&self) -> Color32 {
        self.players[self.get_user_id()].1
    }

    pub fn get_player_by_id(&self, id: &str) -> Option<&(String, Color32)> {
        self.players.get(id)
    }

    fn update_map(&mut self, json: JsonValue) -> Result<(), DraduError> {
        for (id, v) in json.entries() {
            if let Some(obj) = self.map.objects.get_mut(id) {
                match obj {
                    MapObject::Decal(decal) => {
                        if let Ok(pos) = utils::json_to_pos(&v["pos"]) {
                            decal.pos = pos;
                        }
                        if let Some(scale) = v["scale"].as_f32() {
                            decal.scale = scale;
                        }
                    }
                    MapObject::Token(token) => {
                        if let Ok(pos) = utils::json_to_pos(&v["pos"]) {
                            token.pos = pos;
                        }
                        if let Some(scale) = v["scale"].as_f32() {
                            token.scale = scale;
                        }
                    }
                    _ => (),
                }
            } else {
                let path = v["path"].as_str().ok_or(DraduError::ProtocolError)?;
                match v["type"].as_str().ok_or(DraduError::ProtocolError)? {
                    "token" => {
                        self.map.objects.insert(
                            id.to_string(),
                            MapObject::Token(Token {
                                id: id.to_string(),
                                pos: utils::json_to_pos(&v["pos"]).unwrap_or(Pos2::new(0.0, 0.0)),
                                scale: v["scale"].as_f32().unwrap_or(1.0),
                                path: path.to_string(),
                                attributes: v["attributes"].clone(),
                            }),
                        );
                    }
                    "decal" => {
                        self.map.objects.insert(
                            id.to_string(),
                            MapObject::Decal(Decal {
                                id: id.to_string(),
                                pos: utils::json_to_pos(&v["pos"]).unwrap_or(Pos2::new(0.0, 0.0)),
                                scale: v["scale"].as_f32().unwrap_or(1.0),
                                path: path.to_string(),
                            }),
                        );
                    }
                    _ => (),
                }
                if !self.images.contains_key(path) {
                    self.request_file(path);
                }
            }
        }
        Ok(())
    }
    fn update_players(&mut self, json: JsonValue) -> Result<(), DraduError> {
        for (k, v) in json.entries() {
            if v.is_empty() {
                self.players.remove(k);
            } else {
                match self.players.get_mut(k) {
                    Some(player) => {
                        if let Some(s) = v["nickname"].as_str() {
                            player.0 = s.to_string();
                        }
                        if v.has_key("color") {
                            player.1 = utils::color32_from_json_value(&v["color"])?;
                        }
                    }
                    _ => {
                        self.players.insert(
                            k.to_string(),
                            (
                                v["nickname"]
                                    .as_str()
                                    .ok_or(DraduError::ProtocolError)?
                                    .to_string(),
                                utils::color32_from_json_value(&v["color"])?,
                            ),
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn update_chat_log(&mut self, sender_id: String, text: String) {
        self.chat_log.push(ChatMessage { sender_id, text });
    }

    fn update_permissions(&mut self, _json: JsonValue) {}

    fn add_image(&mut self, key: &str, img: RetainedImage) {
        self.images.insert(key.to_string(), img);
    }

    pub fn map(&self) -> &MapState {
        &self.map
    }

    // Will return a placeholder image if key doesn't exist
    pub fn get_image(&self, key: &str) -> &RetainedImage {
        match self.images.get(key) {
            Some(img) => img,
            None => &self.images["placeholder"],
        }
    }
}

pub struct ChatMessage {
    pub sender_id: String,
    pub text: String,
}
