use crate::components::*;
use core::{
    amethyst::{
        core::{shrev::ReaderId, SystemDesc},
        ecs::{
            storage::ComponentEvent, BitSet, ParJoin, Read, ReadStorage, System, SystemData, World,
            WriteStorage,
        },
    },
    components::PropertiesComponent,
    defs::{body::BodyDefinition, body::*, property::*, DefinitionComponent, DefinitionStorage},
    rayon::iter::ParallelIterator,
};

pub mod item_sprites;

// Does sanity checks on the inventory
pub struct BodyUpdatePropertiesSystem {
    body_event_reader_id: ReaderId<ComponentEvent>,

    modified_bodies: BitSet,
    removed_bodies: BitSet,
}
impl<'s> System<'s> for BodyUpdatePropertiesSystem {
    type SystemData = (
        ReadStorage<'s, BodyComponent>,
        WriteStorage<'s, PropertiesComponent>,
        Read<'s, DefinitionStorage<BodyDefinition>>,
    );

    fn run(&mut self, (bodies_storage, mut props_storage, body_defs): Self::SystemData) {
        self.removed_bodies.clear();
        self.modified_bodies.clear();

        for event in bodies_storage
            .channel()
            .read(&mut self.body_event_reader_id)
        {
            match event {
                ComponentEvent::Modified(id) | ComponentEvent::Inserted(id) => {
                    self.modified_bodies.add(*id);
                }
                ComponentEvent::Removed(id) => {
                    self.removed_bodies.add(*id);
                }
            }
        }

        // Remove properties from all removed bodies
        (&bodies_storage, &mut props_storage, &self.removed_bodies)
            .par_join()
            .for_each(|(body, props, _)| props.clear_category(PropertyCategory::Body));

        (&bodies_storage, &mut props_storage, &self.modified_bodies)
            .par_join()
            .for_each(|(body, props_container, _)| {
                props_container.clear_category(PropertyCategory::Body);
                log::trace!("populating properties for new body");

                let def = body.fetch_def(&body_defs).unwrap();

                // TOOD: more detailed
                // For now, we just iterate all part layers and combine all the flags to a single flag
                let mut flags = PartFlags::default();
                def.part_graph
                    .as_ref()
                    .unwrap()
                    .raw_nodes()
                    .iter()
                    .for_each(|part| {
                        part.weight.layers.iter().for_each(|layer| {
                            flags |= layer.flags;
                        });
                    });

                if flags.contains(PartFlags::FineMotor) {
                    props_container.insert(Property::Manipulate(ManipulateFlags::Any));
                }
                if flags.contains(PartFlags::Stance) {
                    props_container.insert(Property::Movement(MovementFlags::Walk));
                }

                // Debug, just add them:
                props_container.insert(Property::Manipulate(ManipulateFlags::Any));
                props_container.insert(Property::Movement(MovementFlags::Walk));

                props_container.insert(Property::MovementSpeed(1000));

                log::trace!("Final: {:?}", props_container);
            });
    }
}

#[derive(Default)]
pub struct BodyUpdatePropertiesSystemDesc;
impl<'a, 'b> SystemDesc<'a, 'b, BodyUpdatePropertiesSystem> for BodyUpdatePropertiesSystemDesc {
    fn build(self, world: &mut World) -> BodyUpdatePropertiesSystem {
        log::trace!("Setup BodyUpdatePropertiesSystem");
        <BodyUpdatePropertiesSystem as System<'_>>::SystemData::setup(world);

        let body_event_reader_id = WriteStorage::<BodyComponent>::fetch(world).register_reader();

        BodyUpdatePropertiesSystem {
            body_event_reader_id,
            removed_bodies: BitSet::new(),
            modified_bodies: BitSet::new(),
        }
    }
}
