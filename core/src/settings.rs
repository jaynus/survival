use crate::num_derive::FromPrimitive;
use amethyst::{assets::Handle, core::math::Vector3, renderer::SpriteSheet};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(
    FromPrimitive, Debug, Copy, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
#[repr(u8)]
pub enum RegionMapRenderMode {
    Normal = 0,
    Pathing = 1,
}
impl Default for RegionMapRenderMode {
    fn default() -> Self {
        RegionMapRenderMode::Normal
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphicsSettings {
    pub tile_dimensions: Vector3<u32>,

    pub tilesets: Vec<(String, PathBuf)>,

    #[serde(skip)]
    pub sprite_sheets: HashMap<String, Handle<SpriteSheet>>,

    pub map_render_mode: RegionMapRenderMode,
}
impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            tile_dimensions: Vector3::new(16, 16, 1),
            tilesets: Vec::new(),
            sprite_sheets: HashMap::default(),
            map_render_mode: RegionMapRenderMode::default(),
        }
    }
}
