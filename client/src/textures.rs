// This module includes built-in textures (Icons, etc.). See instructions on
// how to manipulate them in /images/convert.sh

use eframe::egui;
use egui::{ColorImage, Context, TextureFilter, TextureHandle};

use std::collections::HashMap;
use std::ops::Index;

#[derive(Clone)]
pub struct Textures {
    ctx: Context,
    general_textures: HashMap<String, TextureHandle>,
    light_textures: HashMap<String, TextureHandle>,
    dark_textures: HashMap<String, TextureHandle>,
}

impl Textures {
    pub fn new(ctx: &Context) -> Self {
        let general_textures = load_textures(ctx);
        let (light_textures, dark_textures) = load_themed_textures(ctx);
        Textures {
            ctx: ctx.clone(),
            general_textures,
            light_textures,
            dark_textures,
        }
    }
}

impl Index<&str> for Textures {
    type Output = TextureHandle;

    // Panics if key is not present. If it's a themed texture, will return
    // light or dark mode variant depending on current theme
    fn index(&self, key: &str) -> &Self::Output {
        match self.general_textures.get(key) {
            Some(handle) => handle,
            None => match self.ctx.style().visuals.dark_mode {
                true => &self.dark_textures[key],
                false => &self.light_textures[key],
            },
        }
    }
}

#[allow(unused)]
macro_rules! add_texture {
    ($ctx:expr, $map:expr, $name:expr, $res:expr) => {{
        $map.insert(
            $name.to_string(),
            $ctx.load_texture(
                $name,
                ColorImage::from_rgba_unmultiplied(
                    $res,
                    include_bytes!(concat!("../assets/img/", $name, ".rgba")),
                ),
                TextureFilter::Linear,
            ),
        )
    }};
}

macro_rules! add_themed_texture {
    ($ctx:expr, $maps:expr, $name:expr, $res:expr) => {{
        $maps.0.insert(
            $name.to_string(),
            $ctx.load_texture(
                $name,
                ColorImage::from_rgba_unmultiplied(
                    $res,
                    include_bytes!(concat!("../assets/img/light/", $name, ".rgba")),
                ),
                TextureFilter::Linear,
            ),
        );
        $maps.1.insert(
            $name.to_string(),
            $ctx.load_texture(
                $name,
                ColorImage::from_rgba_unmultiplied(
                    $res,
                    include_bytes!(concat!("../assets/img/dark/", $name, ".rgba")),
                ),
                TextureFilter::Linear,
            ),
        )
    }};
}

fn load_textures(ctx: &Context) -> HashMap<String, TextureHandle> {
    let mut map = HashMap::new();

    add_texture!(ctx, map, "resize", [32, 32]);

    map
}

fn load_themed_textures(
    ctx: &Context,
) -> (
    HashMap<String, TextureHandle>,
    HashMap<String, TextureHandle>,
) {
    let mut maps = (HashMap::new(), HashMap::new());

    add_themed_texture!(ctx, maps, "chat", [32, 32]);
    add_themed_texture!(ctx, maps, "images", [32, 32]);
    add_themed_texture!(ctx, maps, "info", [32, 32]);
    add_themed_texture!(ctx, maps, "send", [32, 32]);
    add_themed_texture!(ctx, maps, "settings", [32, 32]);
    add_themed_texture!(ctx, maps, "tools", [32, 32]);

    maps
}
