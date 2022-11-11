use egui_extras::RetainedImage;

use json::JsonValue;

use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::string::ToString;

use crate::net::PROTOCOL_VERSION;

pub struct Message {
    msg_type: MsgType,
    props: HashMap<String, String>,
    body: Option<MsgBody>,
}

impl Message {
    pub fn new(msg_type: MsgType) -> Self {
        Message {
            msg_type,
            props: HashMap::new(),
            body: None,
        }
    }

    pub fn msg_type(&self) -> MsgType {
        self.msg_type
    }

    // Note that this automatically sets contentLength property
    pub fn into_bytes(self) -> Vec<u8> {
        let mut string = format!("dradu/{} {}\n", PROTOCOL_VERSION, self.msg_type);
        for (key, val) in self.props.iter() {
            string.push_str(&format!("{}:{}\n", key, val));
        }
        let mut body_bytes = match self.body {
            Some(body) => {
                if !self.props.contains_key("contentType") {
                    string.push_str(&format!("contentType:{}\n", body.content_type()));
                }
                body.into_bytes()
            }
            None => Vec::new(),
        };
        string.push_str(&format!("contentLength:{}\n", body_bytes.len()));
        string.push_str("\n");

        let mut bytes = string.into_bytes();
        bytes.append(&mut body_bytes);
        bytes
    }

    pub fn get_prop(&self, prop: &str) -> Option<&str> {
        match self.props.get(prop) {
            Some(s) => Some(&s),
            None => None,
        }
    }

    pub fn set_prop(mut self, key: &str, val: &str) -> Self {
        self.props.insert(key.to_string(), val.to_string());
        self
    }

    pub fn take_body(&mut self) -> Option<MsgBody> {
        self.body.take()
    }

    pub fn attach_body(&mut self, body: MsgBody) {
        self.body = Some(body);
    }
}

// This only parses the *header*. You'll have to use .attach_body() to add
// content into the message
impl FromStr for Message {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.trim().lines();
        // Parsing the heading
        let first_line = lines.next().ok_or(())?;
        let msg_type = match first_line.split_once('/') {
            Some(("dradu", s)) => match s.split_once(' ') {
                Some((ver, msg_type)) if crate::utils::is_compatible_ver(ver)? => {
                    match MsgType::from_str(msg_type) {
                        Ok(s) => s,
                        Err(_) => return Err(()),
                    }
                }
                _ => return Err(()),
            },
            _ => return Err(()),
        };

        // Parsing properties
        let mut props = HashMap::new();
        for l in lines {
            match l.split_once(':') {
                Some((k, v)) => props.insert(k.to_string(), v.trim_start().to_string()),
                None => break,
            };
        }

        Ok(Message {
            msg_type,
            props,
            body: None,
        })
    }
}

// Image variant can't be converted into bytes (egui restriction). If you
// are sending a message, use Bin instead and set "contentType" prop manually
pub enum MsgBody {
    Json(JsonValue),
    Image(RetainedImage),
    Bin(Vec<u8>),
    Text(String),
}

impl MsgBody {
    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Self::Bin(b) => b,
            Self::Json(json) => json.to_string().into_bytes(),
            Self::Text(t) => t.into_bytes(),
            Self::Image(_) => panic!("Can't convert MsgBody::Image into bytes"),
        }
    }

    pub fn retained_image(bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        Ok(Self::Image(RetainedImage::from_image_bytes("", bytes)?))
    }

    pub fn content_type(&self) -> &str {
        match self {
            MsgBody::Json(_) => "json",
            MsgBody::Text(_) => "text",
            MsgBody::Image(_) => "image",
            MsgBody::Bin(_) => "bin",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, strum_macros::Display, strum_macros::EnumString)]
#[strum(ascii_case_insensitive)]
pub enum MsgType {
    Join,
    Init,
    Quit,
    Map,
    Player,
    Msg,
    Perm,
    File,
    Err,
    Synced,
    Ok,
}

#[cfg(test)]
mod tests {
    use crate::net::{Message, MsgType};
    use std::collections::HashMap;
    use std::str::FromStr;

    #[test]
    fn message_into_bytes() {
        let mut props = HashMap::new();
        props.insert("userId".to_string(), "123".to_string());
        let msg = Message {
            msg_type: MsgType::File,
            props,
            body: None,
        };
        let bytes = b"dradu/0.1 File\ncontentLength:0\nuserId:123\n\n\n";
        let bytes2 = b"dradu/0.1 File\nuserId:123\ncontentLength:0\n\n\n";
        let msg_bytes = msg.into_bytes();
        assert!(msg_bytes == bytes || msg_bytes == bytes2);
    }

    #[test]
    fn message_from_str() {
        let string = "dradu/0.1 File\ncontentLength:0\nuserId:123\n\n\n";
        let msg = Message::from_str(string).unwrap();
        assert_eq!(msg.msg_type, MsgType::File);
        assert_eq!(msg.props["contentLength"], "0");
        assert_eq!(msg.props["userId"], "123");
    }
}
