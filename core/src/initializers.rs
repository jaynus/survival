use crate::{
    components::{FoliageComponent, TilePosition, TypeTagComponent},
    defs::{
        foliage::FoliageDefinition, sprites::SpriteOntoFlags, DefinitionStorage, HasProperties,
        Named,
    },
    tiles::region::RegionTile,
};
use amethyst::{
    core::{components::Transform, math::Point3},
    ecs::{world::Builder, Entity, Join, World, WorldExt},
    renderer::Transparent,
    tiles::{Map, TileMap},
};

pub fn spawn_foliage(name: &str, position: &Point3<u32>, world: &mut World) -> Entity {
    let transform = tile_to_transform(position, world);
    // transform.prepend_rotation_z_axis(0.523599);

    log::trace!(
        "Spawning foliage: '{}' @ tile={:?}, world={:?}",
        name,
        position,
        transform.translation()
    );

    let (foliage_component, properties) = {
        let foliage_storage = world.fetch::<DefinitionStorage<FoliageDefinition>>();
        let def = foliage_storage.find(name).unwrap();
        (
            FoliageComponent::new(def.id().unwrap(), &foliage_storage),
            def.default_properties(),
        )
    };

    let entity = world
        .create_entity()
        .with(Transparent)
        .with(foliage_component)
        .with(properties)
        .with(TypeTagComponent::Foliage)
        .with(TilePosition::default())
        .with(transform)
        .build();

    let sprite_ref = world
        .fetch::<DefinitionStorage<FoliageDefinition>>()
        .find(name)
        .unwrap()
        .sprite
        .clone();

    sprite_ref.onto_entity(
        entity,
        world,
        crate::z_level_modifiers::FOLIAGE,
        SpriteOntoFlags::All,
    );

    entity
}

pub fn tile_to_transform(position: &Point3<u32>, world: &World) -> Transform {
    let tilemaps = &world.read_component::<TileMap<RegionTile>>();
    let map = (tilemaps).join().next().unwrap();

    let mut transform = Transform::default();
    transform.set_translation(map.to_world(position));
    transform
}
