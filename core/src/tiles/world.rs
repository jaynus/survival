use amethyst::{core::math::Point3, ecs::World, renderer::palette::Srgba, tiles::Tile};
use bitflags::*;
use bitflags_serial;

bitflags_serial! {
    pub struct WorldTileFlags: u16 {
        const  HasBuilding = 1;
        const  HasZTransition = 1 << 1;
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Biome {
    Glacial,
    Tundra,
    BorealForest,
    ColdDesert,
    TemperateGrassland,
    TemperateDeciduousForest,
    WarmDesert,
    TropicalGrassland,
    Savanna,
    TropicalDeciduousForest,
    TropicalRainForest,
}
impl Default for Biome {
    fn default() -> Self {
        Biome::TemperateDeciduousForest
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct WorldTile {
    pub height: u8,
    pub temperature: u8,
    pub moisture: u8,
    pub flags: WorldTileFlags,
    pub biome: Biome,
}
impl Default for WorldTile {
    fn default() -> Self {
        Self {
            height: 255 / 2,
            temperature: 255 / 2,
            moisture: 255 / 2,
            flags: WorldTileFlags::empty(),
            biome: Biome::default(),
        }
    }
}

impl WorldTile {
    fn world_tile_to_sprite(&self, _: &Point3<u32>, _: &World) -> sprites::SpriteEntry {
        if self.height < 85 {
            sprites::water()
        } else if self.height >= 130 && self.height < 150 {
            sprites::hill()
        } else if self.height >= 150 {
            sprites::mountain()
        } else {
            sprites::grass()
        }
    }
}

impl Tile for WorldTile {
    fn sprite(&self, point: Point3<u32>, world: &World) -> Option<usize> {
        self.world_tile_to_sprite(&point, world).sprite
    }
    fn tint(&self, point: Point3<u32>, world: &World) -> Srgba {
        self.world_tile_to_sprite(&point, world).color
    }
}

pub mod sprites {
    use amethyst::renderer::palette::Srgba;

    pub struct SpriteEntry {
        pub sprite: Option<usize>,
        pub color: Srgba,
    }

    pub fn water() -> SpriteEntry {
        SpriteEntry {
            sprite: Some(126),
            color: Srgba::new(0.0, 0.0, 1.0, 1.0),
        }
    }

    pub fn mountain() -> SpriteEntry {
        SpriteEntry {
            sprite: Some(225),
            color: Srgba::new(0.41, 0.41, 0.41, 1.0),
        }
    }

    pub fn hill() -> SpriteEntry {
        SpriteEntry {
            sprite: Some(254),
            color: Srgba::new(0.823, 0.411, 0.117, 1.0),
        }
    }

    pub fn grass() -> SpriteEntry {
        SpriteEntry {
            sprite: Some(27),
            color: Srgba::new(0.0, 1.0, 0.0, 1.0),
        }
    }
}
