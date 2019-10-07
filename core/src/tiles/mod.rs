use crate::{
    defs::{
        material::{MaterialDefinition, MaterialLayerRef, MaterialLayerRefCompact, MaterialState},
        DefinitionStorage,
    },
    shrinkwraprs::Shrinkwrap,
    tiles::region::RegionTile,
};
use amethyst::{
    core::{
        geometry::Plane,
        math::{Point2, Point3, Vector2, Vector3},
        Transform,
    },
    ecs::{BitSet, Entities, Join, Read, ReadExpect, ReadStorage, SystemData, World},
    renderer::{ActiveCamera, Camera},
    tiles::{iters::Region, CoordinateEncoder, DrawTiles2DBounds, Map, Tile, TileMap},
    window::ScreenDimensions,
};
use std::collections::HashMap;

pub mod region;
pub mod world;

pub const TILE_SCALE: u64 = 1000;

#[derive(Default)]
pub struct CurrentTileZ(pub u32, pub (f32, f32));

#[derive(Shrinkwrap, Default, Clone)]
#[shrinkwrap(mutable)]
pub struct TileEntityStorage(pub HashMap<u32, BitSet>);
impl TileEntityStorage {
    pub fn get_point(&self, point: &Point3<u32>, map: &TileMap<RegionTile>) -> Option<&BitSet> {
        self.0.get(&map.encode(point).unwrap())
    }
}

pub fn distance(one: Point3<u32>, two: Point3<u32>) -> u32 {
    use amethyst::core::math::convert;

    amethyst::core::math::distance(
        &convert::<Point3<u32>, Point3<f32>>(one),
        &convert::<Point3<u32>, Point3<f32>>(two),
    ) as u32
}

#[derive(Default, Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct LayerBits {
    materials: [u16; 4],
    states: [MaterialState; 4],
    damage: u8,
}
impl LayerBits {
    pub fn fill_compact(mut self, compact: &MaterialLayerRefCompact) -> Self {
        (0..4).for_each(|n| {
            self.set_material(n, compact.material_id);
            self.set_state(n, compact.state);
        });
        self
    }

    pub fn from_material_refs_compact(refs: &[MaterialLayerRefCompact]) -> Self {
        assert!(refs.len() < 5);
        let mut ret = LayerBits::default();

        refs.iter().enumerate().for_each(|(n, compact)| {
            ret.set_material(n, compact.material_id);
            ret.set_state(n, compact.state);
            // ret.set_damage(n, layer.2);
        });

        ret
    }

    pub fn from_material_refs(
        refs: &[MaterialLayerRef],
        defs: &DefinitionStorage<MaterialDefinition>,
    ) -> Self {
        assert!(refs.len() < 5);
        let mut ret = LayerBits::default();

        refs.iter().enumerate().for_each(|(n, layer)| {
            let compact = layer.to_compact(defs);
            ret.set_material(n, compact.material_id);
            ret.set_state(n, compact.state);
            // ret.set_damage(n, layer.2);
        });

        ret
    }

    pub fn len(&self) -> usize {
        self.materials
            .iter()
            .fold(0, |acc, m| if *m == 0 { acc } else { acc + 1 })
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn material(&self, layer_number: usize) -> u32 {
        assert!(layer_number < 4);
        u32::from(self.materials[layer_number])
    }

    #[inline]
    pub fn state(&self, layer_number: usize) -> MaterialState {
        assert!(layer_number < 4);
        self.states[layer_number]
    }

    #[inline]
    pub fn set_material(&mut self, layer_number: usize, id: u32) {
        assert!(layer_number < 4);
        self.materials[layer_number] = id as u16;
    }

    #[inline]
    pub fn set_state(&mut self, layer_number: usize, state: MaterialState) {
        assert!(layer_number < 4);
        self.states[layer_number] = state;
    }
}

#[derive(Default, Debug)]
pub struct DrawRegionTileBounds;
impl DrawTiles2DBounds for DrawRegionTileBounds {
    fn bounds<T: Tile, E: CoordinateEncoder>(map: &TileMap<T, E>, world: &World) -> Region {
        let camera_fetch =
            amethyst::renderer::submodules::gather::CameraGatherer::gather_camera_entity(world);
        assert!(camera_fetch.is_some());

        let (entities, active_camera, screen_dimensions, transforms, cameras, current_tile_z) =
            <(
                Entities<'_>,
                Read<'_, ActiveCamera>,
                ReadExpect<'_, ScreenDimensions>,
                ReadStorage<'_, Transform>,
                ReadStorage<'_, Camera>,
                Read<'_, CurrentTileZ>,
            )>::fetch(world);

        //let camera_tile_id = entity_tile_ids.get(camera_entity).u wrap();
        let mut camera_join = (&cameras, &transforms).join();
        if let Some((camera, camera_transform)) = active_camera
            .entity
            .and_then(|a| camera_join.get(a, &entities))
            .or_else(|| camera_join.next())
        {
            let current_z = current_tile_z.0 as f32 * map.tile_dimensions().z as f32;

            // Shoot a ray at each corner of the camera, and determine what tile it hits at the target
            // Z-level
            let proj = camera.projection();
            let plane = Plane::with_z(current_z);

            let ray = proj.screen_ray(
                Point2::new(0.0, 0.0),
                Vector2::new(screen_dimensions.width(), screen_dimensions.height()),
                camera_transform,
            );
            let top_left = ray.at_distance(ray.intersect_plane(&plane).unwrap());

            let ray = proj.screen_ray(
                Point2::new(screen_dimensions.width(), screen_dimensions.height()),
                Vector2::new(screen_dimensions.width(), screen_dimensions.height()),
                camera_transform,
            );
            let bottom_right = ray.at_distance(ray.intersect_plane(&plane).unwrap()).coords
                + Vector3::new(
                    map.tile_dimensions().x as f32 * 5.0,
                    -(map.tile_dimensions().y as f32 * 5.0),
                    0.0,
                );

            let half_dimensions = Vector3::new(
                (map.tile_dimensions().x * map.dimensions().x) as f32 / 2.0,
                (map.tile_dimensions().x * map.dimensions().y) as f32 / 2.0,
                (map.tile_dimensions().x * map.dimensions().z) as f32 / 2.0,
            );
            let bottom_right = Point3::new(
                bottom_right
                    .x
                    .min(half_dimensions.x - map.tile_dimensions().x as f32)
                    .max(-half_dimensions.x),
                bottom_right
                    .y
                    .min(half_dimensions.y - map.tile_dimensions().y as f32)
                    .max(-half_dimensions.y + map.tile_dimensions().y as f32),
                bottom_right
                    .z
                    .min(half_dimensions.z - map.tile_dimensions().z as f32)
                    .max(-half_dimensions.z),
            );

            let min = map
                .to_tile(&top_left.coords)
                .unwrap_or_else(|| Point3::new(0, 0, current_tile_z.0));

            let max = map.to_tile(&bottom_right.coords).unwrap_or_else(|| {
                Point3::new(
                    map.dimensions().x - 1,
                    map.dimensions().y - 1,
                    current_tile_z.0,
                )
            });
            Region::new(min, max)
        } else {
            Region::empty()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_bits() {
        let mut bits = LayerBits::default();

        let layer1 = 123;
        let layer2 = 666;
        let layer3 = 999;
        let layer4 = 0;

        bits.set_material(0, layer1);
        assert_eq!(bits.material(0), layer1);
        assert_eq!(bits.len(), 1);

        bits.set_material(1, layer2);
        assert_eq!(bits.material(1), layer2);
        assert_eq!(bits.len(), 2);

        bits.set_material(2, layer3);
        assert_eq!(bits.material(2), layer3);
        assert_eq!(bits.len(), 3);

        bits.set_material(3, layer4);
        assert_eq!(bits.material(0), layer1);
        assert_eq!(bits.material(1), layer2);
        assert_eq!(bits.material(2), layer3);
        assert_eq!(bits.material(3), layer4);
        assert_eq!(bits.len(), 3);

        bits.set_material(0, 0);
        assert_eq!(bits.len(), 2);
    }
}
