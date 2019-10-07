use core::amethyst::ecs::{World, WorldExt};
use core::defs::DefinitionStorage;
use core::defs::{
    action::ActionDefinition, body::BodyDefinition, building::BuildingDefinition,
    creature::CreatureDefinition, digestion::DigestionDefinition, foliage::FoliageDefinition,
    item::ItemDefinition, material::MaterialDefinition, race::RaceDefinition,
    reaction::ReactionDefinition, InheritDefinitionStorage, Named,
};

pub fn assets(world: &mut World) -> Result<(), failure::Error> {
    log::info!("Loading definitions...");

    world.register::<core::components::SpatialComponent>();

    let mut storage =
        DefinitionStorage::<MaterialDefinition>::from_folder("resources/defs/materials")?;
    storage.apply_inherits()?;
    world.insert(storage);

    let mut storage = DefinitionStorage::<BodyDefinition>::from_folder("resources/defs/bodies")?;
    storage.apply_inherits()?;
    world.insert(storage);

    let mut storage =
        DefinitionStorage::<DigestionDefinition>::from_folder("resources/defs/digestion")?;
    storage.apply_inherits()?;
    world.insert(storage);

    world.insert(DefinitionStorage::<RaceDefinition>::from_folder(
        "resources/defs/races",
    )?);

    world.insert(DefinitionStorage::<CreatureDefinition>::from_folder(
        "resources/defs/creatures",
    )?);

    world.insert(DefinitionStorage::<ActionDefinition>::from_folder(
        "resources/defs/actions",
    )?);

    world.insert(DefinitionStorage::<ReactionDefinition>::from_folder(
        "resources/defs/reactions",
    )?);

    world.insert(DefinitionStorage::<BuildingDefinition>::from_folder(
        "resources/defs/buildings",
    )?);

    world.insert(DefinitionStorage::<FoliageDefinition>::from_folder(
        "resources/defs/foliage",
    )?);

    world.insert(DefinitionStorage::<ItemDefinition>::from_folder(
        "resources/defs/items",
    )?);

    validate_defs(world)
}

pub fn reload_defs(world: &mut World) -> Result<(), failure::Error> {
    log::info!("Reloading definitions...");

    /*
    world
        .fetch_mut::<DefinitionStorage<MaterialDefinition>>()
        .reload()?;
    world
        .fetch_mut::<DefinitionStorage<BodyDefinition>>()
        .reload()?;
    world
        .fetch_mut::<DefinitionStorage<ActionDefinition>>()
        .reload()?;
    world
        .fetch_mut::<DefinitionStorage<BuildingDefinition>>()
        .reload()?;
        */
    assets(world)
}

pub fn validate_foliage(world: &World) -> Result<(), failure::Error> {
    // Validate that all materials actually exist
    let foliage = world.fetch::<DefinitionStorage<FoliageDefinition>>();
    let materials = world.fetch::<DefinitionStorage<MaterialDefinition>>();

    println!("Materials: {:?}", *materials);
    println!("find = {:?}", materials.find("oak"));

    for foliage in foliage.iter() {
        for layer in &foliage.material_layers {
            materials.find(&layer.material.name()).ok_or_else(|| {
                failure::format_err!(
                    "Invalid material specified on foliage! name={}, material={}",
                    foliage.name(),
                    layer.material.name()
                )
            })?;
        }
    }

    Ok(())
}

pub fn validate_defs(world: &World) -> Result<(), failure::Error> {
    validate_foliage(world)?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use amethyst::ecs::{World, WorldExt};

    #[test]
    fn validate_definitions() -> Result<(), failure::Error> {
        let mut world = World::new();
        super::assets(&mut world)?;
        super::validate_defs(&world)
    }
}
