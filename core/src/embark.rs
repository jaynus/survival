use amethyst::{core::math::Vector2, tiles::iters::Region};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmbarkSettings {
    pub world_dimensions: Vector2<u32>,
    pub region: Option<Region>,
}

impl Default for EmbarkSettings {
    fn default() -> Self {
        Self {
            world_dimensions: Vector2::new(3, 3),
            region: None,
        }
    }
}
