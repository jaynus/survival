#![allow(clippy::type_repetition_in_bounds)]

use crate::Generator;
use core::{
    amethyst::{
        assets::Handle,
        core::math::{Point3, Vector3},
        ecs::World,
        renderer::sprite::SpriteSheet,
        tiles::{iters::Region, CoordinateEncoder, Map, MapStorage, MortonEncoder2D, TileMap},
    },
    tiles::world::WorldTile,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct WorldMap<E = MortonEncoder2D>
where
    E: CoordinateEncoder,
{
    pub map: TileMap<WorldTile, E>,
}
impl<E> WorldMap<E>
where
    E: CoordinateEncoder,
{
    pub fn new(
        dimensions: Vector3<u32>,
        tile_dimensions: Vector3<u32>,
        sprite_sheet: Option<Handle<SpriteSheet>>,
    ) -> Self {
        Self {
            map: TileMap::new(dimensions, tile_dimensions, sprite_sheet),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StandardSettings<'a> {
    pub heightmap: &'a image::GrayImage,
}
impl<'a> StandardSettings<'a> {
    pub fn new(heightmap: &'a image::GrayImage) -> Self {
        Self { heightmap }
    }
}
pub struct StandardGenerator<'a> {
    settings: StandardSettings<'a>,
}
impl<'a> StandardGenerator<'a> {
    pub fn new(settings: StandardSettings<'a>) -> Self {
        Self { settings }
    }
}
impl<'a> Generator for StandardGenerator<'a> {
    type Tile = WorldTile;

    fn execute<E, R>(
        &mut self,
        map: &mut TileMap<WorldTile, E>,
        _world: &mut World,
        _rng: &mut R,
    ) -> Result<(), failure::Error>
    where
        E: CoordinateEncoder,
        R: core::rand::Rng + Send + Sync + Clone + Sized,
    {
        use image::Pixel;
        // Fill 1-1 pixels with the tilemap

        if map.dimensions().x != self.settings.heightmap.dimensions().0
            || map.dimensions().y != self.settings.heightmap.dimensions().0
        {
            panic!("Unsupported dimensions dfference")
        }

        Region::new(
            Point3::new(0, 0, 0),
            Point3::from(map.dimensions() - Vector3::new(1, 1, 1)),
        )
        .iter()
        .for_each(|coord| {
            map.get_mut(&coord).unwrap().height = self
                .settings
                .heightmap
                .get_pixel(coord.x, coord.y)
                .channels()[0];
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{amethyst::ecs::World, rand::SeedableRng};

    #[test]
    fn from_heightmap() {
        // Load the heightmap image
        let app_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = app_root.join("../external/worldgen/output/dt_spade_interp.png");
        println!("{:?}", path);

        if let image::DynamicImage::ImageLuma8(image) = image::open(path).unwrap() {
            let mut world = World::new();

            let mut world_map = WorldMap::<MortonEncoder2D>::new(
                Vector3::new(1024, 1024, 1),
                Vector3::new(1, 1, 1),
                None,
            );

            let settings = StandardSettings::new(&image);
            let mut generator = StandardGenerator::new(settings);

            let mut rng = core::rand_xorshift::XorShiftRng::from_seed([
                122, 154, 21, 182, 159, 131, 187, 243, 134, 230, 110, 10, 31, 174, 6, 4,
            ]);

            generator
                .execute(&mut world_map.map, &mut world, &mut rng)
                .unwrap();
        } else {
            panic!("Invalid image")
        }
    }
}
