use crate::{
    defs::property::MovementFlags,
    settings::{GraphicsSettings, RegionMapRenderMode},
    tiles::LayerBits,
};
use amethyst::{
    core::{ecs::World, math::Point3},
    renderer::palette::Srgba,
    tiles::Tile,
};
use bitflags::*;
use bitflags_serial;

bitflags_serial! {
    pub struct RegionTileFlags: u16 {
        const  HasBuilding = 1;
        const  HasZTransition = 1 << 1;
    }
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegionTile {
    layers: LayerBits,
    pub flags: RegionTileFlags,
}

impl RegionTile {
    pub fn new(layers: LayerBits) -> Self {
        Self {
            layers,
            //entities: None,
            flags: RegionTileFlags::empty(),
        }
    }

    pub fn passable(&self, flags: MovementFlags) -> bool {
        self.movement_modifier(flags) > 0
    }

    pub fn movement_modifier(&self, _: MovementFlags) -> u32 {
        if self.flags.contains(RegionTileFlags::HasBuilding)
            || self.layers.len() == 4
            || self.layers.is_empty()
        {
            0
        } else {
            1
        }
    }
}

impl Tile for RegionTile {
    fn tint(&self, _: Point3<u32>, world: &World) -> Srgba {
        match world.fetch::<GraphicsSettings>().map_render_mode {
            RegionMapRenderMode::Normal => Srgba::new(1.0, 1.0, 1.0, 1.0),
            RegionMapRenderMode::Pathing => {
                if self.passable(MovementFlags::Walk) {
                    Srgba::new(0.0, 1.0, 0.0, 1.0)
                } else {
                    Srgba::new(1.0, 0.0, 0.0, 1.0)
                }
            }
        }
    }

    fn sprite(&self, _: Point3<u32>, _: &World) -> Option<usize> {
        if self.layers.is_empty() {
            return None;
        }

        if self.layers.len() < 4 {
            Some(11)
        } else {
            Some(15)
        }
    }
}
