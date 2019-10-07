pub mod random;

use crate::{world::WorldMap, Generator};
use core::{
    amethyst::{
        ecs::World,
        tiles::{CoordinateEncoder, TileMap},
    },
    tiles::region::RegionTile,
};

#[derive(Clone, Copy, Debug)]
pub struct StandardSettings {}
impl<'a> Default for StandardSettings {
    fn default() -> Self {
        Self {}
    }
}
pub struct StandardGenerator<'a, E>
where
    E: CoordinateEncoder,
{
    settings: StandardSettings,
    world_map: &'a WorldMap<E>,
    _marker: std::marker::PhantomData<(E)>,
}
impl<'a, E> StandardGenerator<'a, E>
where
    E: CoordinateEncoder,
{
    pub fn new(settings: StandardSettings, world_map: &'a WorldMap<E>) -> Self {
        Self {
            settings,
            world_map,
            _marker: Default::default(),
        }
    }
}
impl<'a, E> Generator for StandardGenerator<'a, E>
where
    E: CoordinateEncoder,
{
    type Tile = RegionTile;

    fn execute<EM, R>(
        &mut self,
        _map: &mut TileMap<RegionTile, EM>,
        _world: &mut World,
        _rng: &mut R,
    ) -> Result<(), failure::Error>
    where
        EM: CoordinateEncoder,
        R: core::rand::Rng + Send + Sync + Clone + Sized,
    {
        unimplemented!()
    }
}
