#![deny(clippy::pedantic, clippy::all)]
#![allow(
    dead_code,
    clippy::default_trait_access,
    clippy::module_name_repetitions
)]
use core::amethyst::{
    ecs::World,
    tiles::{CoordinateEncoder, Tile, TileMap},
};

pub mod region;
pub mod utils;
pub mod world;

pub trait Generator {
    type Tile: Tile;

    fn execute<E, R>(
        &mut self,
        map: &mut TileMap<Self::Tile, E>,
        world: &mut World,
        rng: &mut R,
    ) -> Result<(), failure::Error>
    where
        E: CoordinateEncoder,
        R: core::rand::Rng + Send + Sync + Clone + Sized;
}
