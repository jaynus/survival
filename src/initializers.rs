use crate::components::*;
use amethyst::{
    assets::{AssetStorage, Handle, Loader, ProgressCounter},
    core::{
        components::Transform,
        math::{Point3, Vector3},
        Time, WithNamed,
    },
    ecs::{world::Builder, Entity, Join, ReadStorage, SystemData, World, WorldExt, WriteStorage},
    renderer::{Camera, ImageFormat, SpriteSheet, SpriteSheetFormat, Texture, Transparent},
    tiles::{Map, MapStorage, TileMap},
    window::ScreenDimensions,
};
use core::{
    components::TilePosition,
    defs::{
        body::BodyDefinition,
        building::BuildingDefinition,
        digestion::DigestionDefinition,
        item::ItemDefinition,
        material::{MaterialRef, MaterialState},
        sprites::SpriteOntoFlags,
        DefinitionStorage, HasProperties, Named,
    },
    settings::GraphicsSettings,
    tiles::region::{RegionTile, RegionTileFlags},
};

pub use core::initializers::{self, tile_to_transform};

#[allow(unused_variables)]
pub fn spawn_creature(name: &str, position: &Point3<u32>, world: &mut World) -> Entity {
    unimplemented!()
}

pub fn spawn_pawn(race_name: &str, position: &Point3<u32>, world: &mut World) -> Entity {
    let transform = tile_to_transform(position, world);

    log::trace!(
        "Spawning pawn: @ tile={:?}, world={:?}",
        position,
        transform.translation()
    );

    let (race, body, properties, spatial) = {
        let races = world.fetch::<DefinitionStorage<RaceDefinition>>();
        let bodies = world.fetch::<DefinitionStorage<BodyDefinition>>();
        let _digestions = world.fetch::<DefinitionStorage<DigestionDefinition>>();

        let race = races.find(race_name).unwrap();
        let body = bodies.find(&race.body).unwrap();

        // TODO: just copy the body dimensions for now

        (
            RaceComponent::new(race.id().unwrap()),
            BodyComponent::new(body.id().unwrap(), &bodies),
            race.default_properties()
                .merge(PropertiesMergeResolution::Error, &body.default_properties()),
            SpatialComponent::new(body.dimensions.unwrap().mean, body.mass.unwrap().mean),
        )
    };
    let idle = IdleComponent::new(&world.fetch::<Time>());

    let entity = world
        .create_entity()
        .with(PawnComponent::default())
        .with(idle)
        .with(spatial)
        .with(properties)
        .with(AttributesComponent::default())
        .with(PersonalityComponent::default())
        .with(PyscheNeedsComponent::default())
        .with(TypeTagComponent::Pawn(PawnType::Player))
        .with(Transparent)
        .with(body)
        .with(race)
        .with(TilePosition::default())
        .with(transform)
        .build();

    let sprite_ref = {
        world
            .fetch::<DefinitionStorage<BodyDefinition>>()
            .find(
                &world
                    .fetch::<DefinitionStorage<RaceDefinition>>()
                    .find(race_name)
                    .unwrap()
                    .body,
            )
            .unwrap()
            .sprite
            .clone()
    };

    sprite_ref.as_ref().unwrap().onto_entity(
        entity,
        world,
        core::z_level_modifiers::PAWN,
        SpriteOntoFlags::All,
    );

    entity
}

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn spawn_building(name: &str, position: &Point3<u32>, world: &mut World) -> Entity {
    let transform = tile_to_transform(position, world);

    log::trace!(
        "Spawning building: '{}' @ tile={:?}, world={:?}",
        name,
        position,
        transform.translation()
    );

    let (building_component, properties, spatial) = {
        let buildings = world.fetch::<DefinitionStorage<BuildingDefinition>>();
        let def_id = buildings.get_id(name).unwrap();
        let def = buildings.get(def_id).unwrap();

        (
            BuildingComponent::new(def_id, &buildings),
            def.default_properties(),
            SpatialComponent::new(def.dimensions, 100_000),
        )
    };

    // Set the tiles of the building to containing this building for pathfinding
    {
        let mut map_storage = <(WriteStorage<'_, TileMap<RegionTile>>)>::fetch(world);
        let map = (&mut map_storage).join().next().unwrap();
        // We need to iterate the region of the dimensions, and set them to buildings
        spatial.occupies_tiles(position).iter().for_each(|coord| {
            map.get_mut(&coord)
                .unwrap()
                .flags
                .insert(RegionTileFlags::HasBuilding);
        });
    }

    let entity = world
        .create_entity()
        .with(Transparent)
        .with(building_component)
        .with(properties)
        .with(spatial)
        .with(TypeTagComponent::Building)
        .with(TilePosition::default())
        .with(transform)
        .build();

    let sprite_ref = world
        .fetch::<DefinitionStorage<BuildingDefinition>>()
        .find(name)
        .unwrap()
        .sprite
        .clone();

    sprite_ref.onto_entity(
        entity,
        world,
        core::z_level_modifiers::BUILDING,
        SpriteOntoFlags::All,
    );

    entity
}

pub fn spawn_item_world(
    name: &str,
    position: Option<Transform>,
    material: Option<MaterialRef>,
    parent: Option<ItemParentComponent>,
    properties: Option<PropertiesComponent>,
    world: &mut World,
) -> Entity {
    assert!(position.is_none() || parent.is_none());

    let transform = if parent.is_none() {
        position.unwrap()
    } else {
        let transform_storage = <(ReadStorage<'_, Transform>)>::fetch(&world);
        let parent_transform = transform_storage.get(parent.as_ref().unwrap().parent);

        if let Some(position) = position {
            position
        } else if let Some(t) = parent_transform {
            t.clone()
        } else {
            panic!("Impossible case")
        }
    };

    log::trace!(
        "Spawning item: '{}' @ tile={:?}, world={:?}",
        name,
        {
            let tilemaps = &world.read_component::<TileMap<RegionTile>>();
            let map = (tilemaps).join().next().unwrap();
            map.to_tile(transform.translation()).unwrap()
        },
        transform.translation()
    );

    let (item, properties, spatial) = {
        let def_storage = world.fetch::<DefinitionStorage<ItemDefinition>>();
        let def_id = def_storage.get_id(name).unwrap();
        let def = def_storage.get(def_id).unwrap();

        let mut return_properties = def.default_properties();

        if let Some(properties) = properties {
            return_properties.extend(properties.iter());
        }

        let material = if let Some(material) = material {
            material
        } else {
            MaterialRef::new(&"oak", MaterialState::Solid)
        };

        (
            ItemComponent::new(def_id, &material, &def_storage),
            return_properties,
            SpatialComponent::new(def.dimensions.unwrap(), 100),
        )
    };

    let mut builder = world
        .create_entity()
        .with(transform)
        .with(item)
        .with(TypeTagComponent::Item)
        .with(Transparent)
        .with(spatial)
        .with(TilePosition::default())
        .with(properties);

    if let Some(parent) = parent {
        builder = builder.with(parent);
    }

    builder.build()
}

pub fn spawn_item(
    name: &str,
    position: Option<Point3<u32>>,
    material: Option<MaterialRef>,
    parent: Option<ItemParentComponent>,
    properties: Option<PropertiesComponent>,
    world: &mut World,
) -> Entity {
    spawn_item_world(
        name,
        position.map(|p| tile_to_transform(&p, world)),
        material,
        parent,
        properties,
        world,
    )
}

pub fn map(world: &mut World, _: &str, _: &mut ProgressCounter) -> Entity {
    let sprite_sheet = {
        world
            .read_resource::<GraphicsSettings>()
            .sprite_sheets
            .get("default_map")
            .map(|v| (*v).clone())
    };

    let map = TileMap::<RegionTile>::new(
        Vector3::new(256, 256, 256),
        Vector3::new(16, 16, 1),
        sprite_sheet,
    );

    world
        .create_entity()
        .with(map)
        .with(Transform::default())
        .build()
}

pub fn load_sprite_sheet(
    world: &mut World,
    png_path: &str,
    ron_path: &str,
    counter: &mut ProgressCounter,
) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(png_path, ImageFormat::default(), (), &texture_storage)
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        ron_path,
        SpriteSheetFormat(texture_handle),
        counter,
        &sprite_sheet_store,
    )
}

pub fn camera(world: &mut World, _: &mut ProgressCounter) -> Entity {
    let (width, height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
    };
    //log::trace!("Init camera with dimensions: {}x{}", width, height);

    let mut camera_transform = Transform::default();
    camera_transform.set_translation_z(0.99);

    world
        .create_entity()
        .with(camera_transform)
        .with(Camera::standard_2d(width, height))
        //.with(Camera::standard_3d(width, height))
        .named("camera")
        .build()
}
