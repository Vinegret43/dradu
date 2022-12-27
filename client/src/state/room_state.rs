use eframe::egui;
use egui::{Color32, Context, Pos2};
use egui_extras::RetainedImage;

use json::{object, JsonValue};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;

use crate::fs::AssetDirHandler;
use crate::net::{Connection, LoopbackConnection, Message, MsgBody, MsgType, ServerConnection};
use crate::state::map::{MapObject, MapState};
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
    connection: Box<dyn Connection>,
    fs: AssetDirHandler,

    players: HashMap<String, (String, Color32)>, // Id: (Nickname, Color)
    // It promises that all images referenced in map *will* be here,
    // however, there is a placeholder image which will be returned otherwise
    images: HashMap<String, RetainedImage>,
    map: MapState,
}

impl<'a> RoomState {
    pub fn join_room(addr: SocketAddr, room_id: &str, ctx: &Context) -> Result<Self, DraduError> {
        let connection = ServerConnection::join_room(addr, room_id, ctx)?;
        Ok(Self::with_connection(Box::new(connection), false))
    }

    pub fn create_new_room(addr: SocketAddr, ctx: &Context) -> Result<Self, DraduError> {
        let connection = ServerConnection::create_new_room(addr, ctx)?;
        Ok(Self::with_connection(Box::new(connection), true))
    }

    pub fn create_local_server(ctx: &Context) -> Self {
        let connection = LoopbackConnection::new(ctx);
        Self::with_connection(Box::new(connection), true)
    }

    fn with_connection(connection: Box<dyn Connection>, master: bool) -> Self {
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
            match (msg.msg_type(), msg.take_body()) {
                (MsgType::Map, Some(MsgBody::Json(json))) => {
                    self.update_map(json)?;
                }
                (MsgType::Player, Some(MsgBody::Json(json))) => {
                    self.update_players(json)?;
                }
                (MsgType::Msg, Some(MsgBody::Text(text))) => {
                    if let Some(user_id) = msg.get_prop("userId") {
                        self.update_chat_log(user_id.to_owned(), text);
                    }
                }
                (MsgType::Perm, Some(MsgBody::Json(json))) => {
                    self.update_permissions(json); // TODO
                }
                (MsgType::File, body) => {
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
                        if let Some(MsgBody::Image(image)) = body {
                            self.add_image(msg.get_prop("path").unwrap_or(""), image)
                        }
                    }
                }
                (MsgType::Synced, _) => {}
                (MsgType::Err, _) => (),
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

    pub fn request_file(&mut self, path: &str) -> Result<(), DraduError> {
        if self.master {
            if !self.images.contains_key(path) {
                let image = self.fs.get_retained_image(path)?;
                self.add_image(path, image);
            }
        } else {
            let msg = Message::new(MsgType::File).set_prop("path", path);
            self.send_msg(msg)?;
        }
        Ok(())
    }

    pub fn quit_room(&mut self) {
        self.send_msg(Message::new(MsgType::Quit));
        self.connection.close()
    }

    pub fn send_chat_message(&mut self, text: &str) {
        let mut msg = Message::new(MsgType::Msg);
        msg.attach_body(MsgBody::Text(text.to_owned()));
        self.send_msg(msg);
    }

    pub fn insert_from_path<P: AsRef<Path>>(
        &mut self,
        path: P,
        obj_type: &str,
    ) -> Result<(), DraduError> {
        let path = path.as_ref();
        let path_str = path.to_str().unwrap();
        if !self.images.contains_key(path_str) {
            let image = self.fs.get_retained_image(path)?;
            self.add_image(path_str, image);
        }
        let mut msg = Message::new(MsgType::Map);
        let inner_json = object! {
            "type": obj_type,
            "path": path_str
        };
        let mut json = JsonValue::new_object();
        json[utils::random_id()] = inner_json;

        msg.attach_body(MsgBody::Json(json));
        self.send_msg(msg);
        Ok(())
    }

    pub fn set_background_image<P: AsRef<Path>>(&mut self, path: P) -> Result<(), DraduError> {
        let path = path.as_ref();
        let path_str = path.to_str().unwrap();
        if !self.images.contains_key(path_str) {
            let image = self.fs.get_retained_image(path)?;
            self.add_image(path_str, image);
        }
        let mut msg = Message::new(MsgType::Map);
        let json = json::object! {
            "background": {
                "path": path_str,
            }
        };
        msg.attach_body(MsgBody::Json(json));
        self.send_msg(msg);
        Ok(())
    }

    pub fn move_map_object(&mut self, id: &str, pos: Pos2) {
        if self.map.objects.contains_key(id) {
            let mut msg = Message::new(MsgType::Map);
            let inner_json = object! {"pos": [pos.x, pos.y]};
            let mut json = JsonValue::new_object();
            json[id] = inner_json;
            msg.attach_body(MsgBody::Json(json));
            self.send_msg(msg);
        }
    }

    pub fn delete_map_object(&mut self, id: &str) {
        if self.map.objects.contains_key(id) {
            let mut msg = Message::new(MsgType::Map);
            let mut json = JsonValue::new_object();
            json[id] = object! {};
            msg.attach_body(MsgBody::Json(json));
            self.send_msg(msg);
        }
    }

    pub fn clear_map(&mut self) {
        let mut msg = Message::new(MsgType::Map);
        msg.attach_body(MsgBody::Json(JsonValue::Null));
        self.send_msg(msg);
    }

    pub fn rescale_map_object(&mut self, id: &str, scale: f32) {
        if self.map.objects.contains_key(id) {
            let mut msg = Message::new(MsgType::Map);
            let inner_json = object! {"scale": scale};
            let mut json = JsonValue::new_object();
            json[id] = inner_json;
            msg.attach_body(MsgBody::Json(json));
            self.send_msg(msg);
        }
    }

    pub fn update_token_property(&mut self, id: &str, key: &str, val: &str) {
        self.change_token_property(id, key, JsonValue::from(val.trim()))
    }

    pub fn remove_token_property(&mut self, id: &str, key: &str) {
        self.change_token_property(id, key, JsonValue::Null);
    }

    fn change_token_property(&mut self, id: &str, key: &str, val: JsonValue) {
        if let Some(MapObject::Token(_)) = self.map.objects.get(id) {
            let key = key.trim();
            if !key.is_empty() {
                let mut msg = Message::new(MsgType::Map);
                let mut inner_json = object! {"properties": {}};
                inner_json["properties"][key] = val;
                let mut json = JsonValue::new_object();
                json[id] = inner_json;
                msg.attach_body(MsgBody::Json(json));
                self.send_msg(msg);
            }
        }
    }

    pub fn change_grid_size(&mut self, size: [u8; 2]) {
        let mut msg = Message::new(MsgType::Map);
        let json = if size[0] >= 2 && size[1] >= 2 {
            object! {
                "grid": {
                    "size": [size[0], size[1]],
                }
            }
        } else {
            object! {
                "grid": {}
            }
        };
        msg.attach_body(MsgBody::Json(json));
        self.send_msg(msg);
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

    pub fn is_master(&self) -> bool {
        self.master
    }

    pub fn get_player_by_id(&self, id: &str) -> Option<&(String, Color32)> {
        self.players.get(id)
    }

    fn update_map(&mut self, json: JsonValue) -> Result<(), DraduError> {
        // Reset entire map
        if json.is_null() {
            self.map = MapState::default();
        }
        for (id, entry) in json.entries() {
            // Firstly checking for special-case scenarios
            if id == "grid" {
                self.update_grid(entry)?;
            } else if id == "background" {
                self.update_background(entry)?;
            } else if entry.is_empty() {
                self.map.objects.remove(id);
            } else {
                // Otherwise just normally updating all the objects
                if let Some(obj) = self.map.objects.get_mut(id) {
                    obj.update_from_json(&entry)?;
                } else {
                    let obj = MapObject::create_from_json(&entry)?;
                    if !self.images.contains_key(obj.path()) {
                        self.request_file(obj.path())?;
                    }
                    self.map.objects.insert(id.to_string(), obj);
                }
            }
        }
        Ok(())
    }

    fn update_grid(&mut self, json: &JsonValue) -> Result<(), DraduError> {
        if json.is_empty() {
            self.map.grid = None;
        } else {
            let size = &json["size"];
            let columns = size[0].as_u8().ok_or(DraduError::ProtocolError)?;
            let rows = size[1].as_u8().ok_or(DraduError::ProtocolError)?;
            self.map.grid = Some([columns, rows]);
        }
        Ok(())
    }

    fn update_background(&mut self, json: &JsonValue) -> Result<(), DraduError> {
        let path = json["path"].as_str().ok_or(DraduError::ProtocolError)?;
        self.map.background_image = Some(String::from(path));
        if !self.images.contains_key(path) {
            self.request_file(path)?;
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
