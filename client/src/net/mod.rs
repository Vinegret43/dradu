mod connection;
mod message;

pub use connection::{Connection, LoopbackConnection, ServerConnection};
pub use message::{Message, MsgBody, MsgType};

pub const PROTOCOL_VERSION: &str = "0.1";
