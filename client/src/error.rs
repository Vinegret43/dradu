use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::mpsc::RecvTimeoutError;

#[derive(Debug)]
pub enum DraduError {
    ProtocolError,
    ConnectionError,
    Io(std::io::Error),
    ChannelDisconnected,
    TimeoutExceeded,
    ProjectDirNotFound,
    InvalidPath,
    ImageLoadError(String),
}

impl Error for DraduError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl Display for DraduError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::ProtocolError => write!(f, "Key error"),
            Self::ConnectionError => write!(f, "Connection error"),
            Self::ChannelDisconnected => write!(f, "Channel disconnected"),
            Self::TimeoutExceeded => write!(f, "Server response timeout exceeded"),
            Self::Io(err) => write!(f, "{}", err),
            _ => Ok(()), // TODO
        }
    }
}

impl From<std::io::Error> for DraduError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<RecvTimeoutError> for DraduError {
    fn from(error: RecvTimeoutError) -> Self {
        match error {
            RecvTimeoutError::Timeout => Self::TimeoutExceeded,
            RecvTimeoutError::Disconnected => Self::ChannelDisconnected,
        }
    }
}
