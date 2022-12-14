use eframe::egui::{Color32, Context};

use json::JsonValue;

use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::str::FromStr;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use crate::net::{Message, MsgBody, MsgType};
use crate::utils;
use crate::DraduError;

const PE: DraduError = DraduError::ProtocolError;

pub trait Connection {
    fn new_messages(&mut self) -> Result<Vec<Message>, DraduError>;
    fn send_msg(&mut self, msg: Message) -> Result<usize, DraduError>;
    fn get_room_address(&self) -> Result<String, DraduError>;
    fn close(&mut self);
    fn get_user_id(&self) -> &str;
    fn get_nickname(&self) -> &str;
    fn get_user_color(&self) -> Color32;
}

pub struct ServerConnection {
    user_id: String,
    user_cookie: String,
    nickname: String,
    user_color: Color32,
    room_id: String,
    stream: TcpStream,
    receiver: Receiver<Message>,
}

impl ServerConnection {
    pub fn join_room(addr: SocketAddr, room_id: &str, ctx: &Context) -> Result<Self, DraduError> {
        let mut stream = TcpStream::connect(addr)?;

        let receiver = spawn_receiving_thread(stream.try_clone().unwrap(), ctx);

        // Setting up connection
        let mut msg = Message::new(MsgType::Join);
        msg.attach_body(MsgBody::Json(JsonValue::from(format!(
            "{{\"roomId\":\"{}\"}}",
            room_id
        ))));
        stream.write_all(&msg.into_bytes())?;

        let mut msg = receiver.recv_timeout(Duration::from_secs(3))?;
        let json = match (msg.msg_type(), msg.take_body()) {
            (MsgType::Ok, Some(MsgBody::Json(json))) => json,
            _ => return Err(DraduError::ConnectionError),
        };

        let user_id = json["userId"].as_str().ok_or(PE)?.to_string();
        let user_cookie = json["userCookie"].as_str().ok_or(PE)?.to_string();
        let nickname = json["nickname"].as_str().unwrap_or("").to_string();
        let user_color = utils::color32_from_json_value(&json["color"]).unwrap_or(Color32::WHITE);

        Ok(ServerConnection {
            user_id,
            user_cookie,
            user_color,
            nickname,
            room_id: room_id.to_string(),
            stream,
            receiver,
        })
    }

    pub fn create_new_room(addr: SocketAddr, ctx: &Context) -> Result<Self, DraduError> {
        let mut stream = TcpStream::connect(addr)?;

        let receiver = spawn_receiving_thread(stream.try_clone().unwrap(), ctx);

        let msg = Message::new(MsgType::Init);
        stream.write_all(&msg.into_bytes())?;

        let mut msg = receiver.recv_timeout(Duration::from_secs(3))?;
        let json = match (msg.msg_type(), msg.take_body()) {
            (MsgType::Ok, Some(MsgBody::Json(json))) => json,
            _ => return Err(DraduError::ConnectionError),
        };

        let user_id = json["userId"].as_str().ok_or(PE)?.to_string();
        let user_cookie = json["userCookie"].as_str().ok_or(PE)?.to_string();
        let nickname = json["nickname"].as_str().unwrap_or("").to_string();
        let user_color = utils::color32_from_json_value(&json["color"]).unwrap_or(Color32::WHITE);
        let room_id = json["roomId"].as_str().ok_or(PE)?.to_string();

        Ok(ServerConnection {
            user_id,
            user_cookie,
            nickname,
            user_color,
            room_id,
            stream,
            receiver,
        })
    }
}

impl Connection for ServerConnection {
    fn new_messages(&mut self) -> Result<Vec<Message>, DraduError> {
        let mut messages = Vec::new();
        loop {
            match self.receiver.try_recv() {
                Ok(msg) => messages.push(msg),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return Err(DraduError::ChannelDisconnected),
            }
        }
        Ok(messages)
    }

    fn send_msg(&mut self, msg: Message) -> Result<usize, DraduError> {
        let msg = msg
            .set_prop("userId", &self.user_id)
            .set_prop("userCookie", &self.user_cookie);
        Ok(self.stream.write(&msg.into_bytes())?)
    }

    fn get_room_address(&self) -> Result<String, DraduError> {
        Ok(format!("{}#{}", self.stream.peer_addr()?, self.room_id))
    }

    fn close(&mut self) {
        #[allow(unused)]
        {
            self.stream.shutdown(Shutdown::Both);
        }
        // ?HACK?: Manually dropping the mpsc channel so .new_messages() will return Err
        self.receiver = mpsc::channel().1;
    }

    fn get_user_id(&self) -> &str {
        &self.user_id
    }

    fn get_nickname(&self) -> &str {
        &self.nickname
    }

    fn get_user_color(&self) -> Color32 {
        self.user_color
    }
}

// A local server used for creating maps. It basically just echoes (Loops back)
// all messages you send
pub struct LoopbackConnection {
    message_queue: Option<Vec<Message>>,
    ctx: Context,
    closed: bool,
}

impl LoopbackConnection {
    pub fn new(ctx: &Context) -> Self {
        Self {
            message_queue: Some(Vec::new()),
            ctx: ctx.clone(),
            closed: false,
        }
    }
}

impl Connection for LoopbackConnection {
    fn new_messages(&mut self) -> Result<Vec<Message>, DraduError> {
        if self.closed {
            Err(DraduError::ChannelDisconnected)
        } else {
            let vec = self.message_queue.take().unwrap();
            self.message_queue = Some(Vec::new());
            Ok(vec)
        }
    }
    fn send_msg(&mut self, msg: Message) -> Result<usize, DraduError> {
        self.message_queue.as_mut().unwrap().push(msg);
        self.ctx.request_repaint();
        Ok(0)
    }
    fn get_room_address(&self) -> Result<String, DraduError> {
        Ok("local".to_string())
    }
    fn get_user_id(&self) -> &str {
        ""
    }
    fn get_nickname(&self) -> &str {
        "Master"
    }
    fn get_user_color(&self) -> Color32 {
        Color32::RED
    }
    fn close(&mut self) {
        self.closed = true;
    }
}

// TODO: Refactor this
fn spawn_receiving_thread(mut stream: TcpStream, ctx: &Context) -> Receiver<Message> {
    let (tx, rx) = mpsc::channel();

    let ctx = ctx.clone();

    thread::spawn(move || loop {
        stream.take_error().unwrap();
        let mut newlines = 0;
        let mut bytes = Vec::new();
        // Reading
        loop {
            match utils::read_byte(&mut stream) {
                Ok(b) if b == b'\n' => {
                    newlines += 1;
                    bytes.push(b);
                    if newlines == 2 {
                        break;
                    }
                }
                Ok(b) => {
                    newlines = 0;
                    bytes.push(b);
                }
                Err(_) => panic!(),
            }
        }
        // Processing and sending through the channel
        let mut msg = Message::from_str(&String::from_utf8(bytes).unwrap()).unwrap();
        match msg.get_prop("contentLength") {
            Some(s) if s != "0" => {
                let len = s.parse::<usize>().unwrap();
                let mut buf = vec![0; len];
                stream.read_exact(&mut buf).unwrap();
                match msg.get_prop("contentType") {
                    Some("json") => {
                        msg.attach_body(MsgBody::Json(
                            json::parse(&std::str::from_utf8(&buf).unwrap()).unwrap(),
                        ));
                    }
                    Some("text") => {
                        msg.attach_body(MsgBody::Text(String::from_utf8_lossy(&buf).to_string()))
                    }
                    Some("image") => {
                        msg.attach_body(MsgBody::retained_image(&buf).unwrap());
                    }
                    _ => (),
                }
            }
            _ => (),
        }
        tx.send(msg).unwrap();
        ctx.request_repaint();
    });

    rx
}
