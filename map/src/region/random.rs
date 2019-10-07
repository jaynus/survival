use crate::Generator;
use core::{
    amethyst::{
        core::math::{Point3, Vector3},
        ecs::World,
        tiles::{CoordinateEncoder, Map, MapStorage, TileMap},
    },
    defs::{material::MaterialDefinition, DefinitionStorage},
    tiles::{region::RegionTile, LayerBits},
};

#[derive(Clone, Copy, Debug)]
pub struct RandomSettings {
    pub set_percent: f32,
}
impl Default for RandomSettings {
    fn default() -> Self {
        Self { set_percent: 0.1 }
    }
}

pub struct RandomGenerator {
    settings: RandomSettings,
}
impl RandomGenerator {
    pub fn new(settings: RandomSettings) -> Self {
        Self { settings }
    }
}

impl Generator for RandomGenerator {
    type Tile = RegionTile;
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::shadow_unrelated
    )]
    fn execute<E, R>(
        &mut self,
        map: &mut TileMap<RegionTile, E>,
        world: &mut World,
        rng: &mut R,
    ) -> Result<(), failure::Error>
    where
        E: CoordinateEncoder,
        R: core::rand::Rng + Send + Sync + Clone + Sized,
    {
        use core::{
            amethyst::tiles::iters::Region, defs::material::MaterialLayerRef,
            rand::distributions::Standard,
        };
        log::trace!("Enter");

        let defs = world.fetch::<DefinitionStorage<MaterialDefinition>>();
        let default_material = MaterialLayerRef::default().to_compact(&defs);

        // Fill everything with flat walkable tiles
        let region = Region::new(
            Point3::new(0, 0, 0),
            Point3::from(*map.dimensions() - Vector3::new(1, 1, 1)),
        );
        region.iter().for_each(|coord| {
            *map.get_mut(&coord).unwrap() =
                RegionTile::new(LayerBits::from_material_refs_compact(&[default_material]));
        });

        // Set 10% of the map as solid tiles, which is layers.len() = 4
        let tiles_count = (map.dimensions().x * map.dimensions().y) as f32;
        let set_count = (tiles_count * self.settings.set_percent) as usize;

        for _ in 0..set_count {
            let x: f32 = rng.sample(Standard);
            let y: f32 = rng.sample(Standard);
            let point = Point3::new(
                (map.dimensions().x as f32 * x) as u32,
                (map.dimensions().y as f32 * y) as u32,
                0,
            );
            *map.get_mut(&point).unwrap() =
                RegionTile::new(LayerBits::default().fill_compact(&default_material));
        }

        // Set 10% of the map as empty tiles
        let tiles_count = (map.dimensions().x * map.dimensions().y) as f32;
        let set_count = (tiles_count * self.settings.set_percent) as usize;

        for _ in 0..set_count {
            let x: f32 = rng.sample(Standard);
            let y: f32 = rng.sample(Standard);

            *map.get_mut(&Point3::new(
                (map.dimensions().x as f32 * x) as u32,
                (map.dimensions().y as f32 * y) as u32,
                0,
            ))
            .unwrap() = RegionTile::default();
        }

        // Create some trees

        Ok(())
    }
}
