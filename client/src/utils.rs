use eframe::egui::{Color32, ColorImage, Pos2};
use egui_extras::RetainedImage;

use json::JsonValue;

use rand::{distributions::Alphanumeric, Rng};

use std::io::Read;
use std::path::{Component, Path};

use crate::net::PROTOCOL_VERSION;
use crate::DraduError;

// Returns a light-purple 128x128 rectangle
pub fn get_placeholder_image() -> RetainedImage {
    let mut arr = [0; 128 * 128 * 4];
    // Setting that annoying purple color
    for i in 0..(128 * 128) {
        arr[i * 4] = 255;
        arr[i * 4 + 2] = 255;
        arr[i * 4 + 3] = 255;
    }
    RetainedImage::from_color_image(
        "placeholder",
        ColorImage::from_rgba_unmultiplied([128, 128], &arr),
    )
}

pub fn color32_from_json_value(val: &JsonValue) -> Result<Color32, DraduError> {
    match val {
        JsonValue::Array(arr) if arr.len() >= 3 => {
            let r = arr[0].as_u8().ok_or(DraduError::ProtocolError)?;
            let g = arr[1].as_u8().ok_or(DraduError::ProtocolError)?;
            let b = arr[2].as_u8().ok_or(DraduError::ProtocolError)?;
            Ok(Color32::from_rgb(r, g, b))
        }
        _ => Err(DraduError::ProtocolError),
    }
}

pub fn parse_color(s: &str) -> Result<[u8; 3], std::num::ParseIntError> {
    let rgb: Vec<&str> = s.splitn(3, ' ').collect();
    Ok([
        rgb.get(0).unwrap_or(&"0").parse::<u8>()?,
        rgb.get(1).unwrap_or(&"0").parse::<u8>()?,
        rgb.get(2).unwrap_or(&"0").parse::<u8>()?,
    ])
}

pub fn rgb_to_string(rgb: [u8; 3]) -> String {
    format!("{} {} {}", rgb[0], rgb[1], rgb[2])
}

pub fn is_compatible_ver(ver: &str) -> Result<bool, ()> {
    Ok(ver.split_once('.').ok_or(())?.0 == PROTOCOL_VERSION.split_once('.').unwrap().0)
}

pub fn read_byte(stream: &mut impl Read) -> std::io::Result<u8> {
    let mut buf = [0];
    stream.read_exact(&mut buf)?;
    Ok(buf[0])
}

pub fn directory_traversal<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref()
        .components()
        .into_iter()
        .any(|x| x == Component::ParentDir)
}

pub fn random_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

// Accepts Json array with two elements
pub fn json_to_pos(json: &JsonValue) -> Result<Pos2, ()> {
    if let [x, y] = json.members().take(2).collect::<Vec<&JsonValue>>()[..2] {
        Ok(Pos2::new(x.as_f32().ok_or(())?, y.as_f32().ok_or(())?))
    } else {
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::directory_traversal;
    #[test]
    fn test_directory_traversal() {
        assert!(directory_traversal("/usr/share/.."));
        assert!(directory_traversal("/usr/share/../"));
        assert!(directory_traversal("/usr/share/../share"));
        assert!(directory_traversal("smth/../smth"));
        assert!(directory_traversal("../smth"));
        assert!(!directory_traversal("proper_path"));
        assert!(!directory_traversal("/proper/path"));
        assert!(!directory_traversal("/proper/path/."));
    }
}
