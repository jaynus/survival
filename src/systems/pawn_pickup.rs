#![allow(unused_variables, unused_mut)]

use crate::components::{
    CurrentActionComponent, ItemParentComponent, ItemParentRelationship, PropertiesComponent,
};
use core::{
    amethyst::{
        core::{SystemDesc, Transform},
        ecs::{Entities, Join, Read, ReadStorage, System, SystemData, World, Write, WriteStorage},
        shrev::{EventChannel, ReaderId},
        tiles::{Map, TileMap},
    },
    fsm::{ActionEvent, ActionStatus, ActionTarget, Event},
    tiles::region::RegionTile,
};

#[derive(Default)]
pub struct PawnPickupItemSystem {
    reader: Option<ReaderId<ActionEvent>>,
}
impl<'s> System<'s> for PawnPickupItemSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, EventChannel<ActionEvent>>,
        ReadStorage<'s, TileMap<RegionTile>>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, PropertiesComponent>,
        WriteStorage<'s, ItemParentComponent>,
        WriteStorage<'s, CurrentActionComponent>,
    );

    fn run(
        &mut self,
        (
            entities,
            events,
            map_storage,
            transform_storage,
            property_storage,
            mut item_parents_storage,
            mut active_action_storage,
        ): Self::SystemData,
    ) {
        //log::trace!("delta={}", time.delta_time().as_millis());
        for action in events.read(self.reader.as_mut().unwrap()) {
            if let Event::Pickup = &action.event {
                log::trace!("pickup action received");
                log::trace!("target = {:?}", action.targets);

                let map = (&map_storage).join().next().unwrap();

                let source_entity = action.source.unwrap();
                let target_entity =
                    if let ActionTarget::Entity(target) = action.targets.as_ref().unwrap()[0] {
                        target
                    } else {
                        panic!()
                    };

                let active = active_action_storage.get_mut(source_entity).unwrap();

                let source_transform = transform_storage.get(source_entity).unwrap();
                let target_transform = transform_storage.get(target_entity).unwrap();

                // Is the item still in range of the destination we went to, for pickup?
                // TODO: pickup range
                if core::tiles::distance(
                    map.to_tile(source_transform.translation()).unwrap(),
                    map.to_tile(target_transform.translation()).unwrap(),
                ) > 1
                {
                    active.status = Ok(ActionStatus::Failure);
                    return;
                }

                // Confirm we still are near the source entity, and it hasnt been picked up. If so, fail
                if let Some(parent) = item_parents_storage.get(source_entity) {
                    active.status = Ok(ActionStatus::Failure);
                    return;
                }

                // Make the entity a child of our entity. item_sprites handles the removal of the sprite
                item_parents_storage
                    .insert(
                        target_entity,
                        ItemParentComponent::new(source_entity, ItemParentRelationship::On),
                    )
                    .unwrap();

                active.status = Ok(ActionStatus::Success);
            }
        }
    }
}

impl<'a, 'b> SystemDesc<'a, 'b, PawnPickupItemSystem> for PawnPickupItemSystem {
    fn build(self, world: &mut World) -> Self {
        <Self as System<'_>>::SystemData::setup(world);
        let reader = Some(Write::<EventChannel<ActionEvent>>::fetch(world).register_reader());

        Self { reader }
    }
}
