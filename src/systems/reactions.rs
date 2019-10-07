use crate::components::{CurrentActionComponent, ItemComponent, PropertiesComponent};
use core::{
    amethyst::{
        core::{SystemDesc, Transform},
        ecs::{
            BitSet, Entities, Join, LazyUpdate, Read, ReadStorage, System, SystemData, World,
            WorldExt, Write, WriteStorage,
        },
        shrev::{EventChannel, ReaderId},
    },
    defs::{
        reaction::{Kind, ReactionDefinition},
        DefinitionStorage,
    },
    fsm::{ActionEvent, ActionStatus, ActionTarget, Event},
    hibitset::BitSetLike,
};
#[derive(Default)]
pub struct ExecuteRactionSystem {
    reader: Option<ReaderId<ActionEvent>>,
    delete: BitSet,
}
impl<'s> System<'s> for ExecuteRactionSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, LazyUpdate>,
        Read<'s, EventChannel<ActionEvent>>,
        Read<'s, DefinitionStorage<ReactionDefinition>>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, ItemComponent>,
        ReadStorage<'s, PropertiesComponent>,
        WriteStorage<'s, CurrentActionComponent>,
    );

    fn run(
        &mut self,
        (
            entities,
            lazy,
            events,
            reaction_storage,
            transform_storage,
            _item_storage,
            props_storage,
            mut current_action_storage,
        ): Self::SystemData,
    ) {
        // New fell tree events coming through
        for action in events.read(self.reader.as_mut().unwrap()) {
            if let Event::ActivateReaction(reaction_name) = &action.event {
                log::trace!("REACTION Got event: {:?}", action);

                let def = reaction_storage.find(&reaction_name).unwrap();

                // TODO: Validate the reaction conditions
                // We can do this with the subjects of the action?
                // TODO: Action should just take a reaction and populate based off its conditions?
                // This would save us double-entering shit
                // This could auto-populate from if action.event == ActivateReaction(...)
                let reagent_entities_map = def
                    .reagents
                    .iter()
                    .filter_map(|reagent| {
                        if reagent.consume {
                            // Reaction
                            let matches = action
                                .targets
                                .as_ref()
                                .unwrap()
                                .iter()
                                .filter(|target| {
                                    if let ActionTarget::Entity(entity) = target {
                                        match &reagent.kind {
                                            Kind::Properties(reagent_properties) => {
                                                if let Some(properties) = props_storage.get(*entity)
                                                {
                                                    reagent_properties.iter().all(|property| {
                                                        properties.contains_value(property)
                                                    })
                                                } else {
                                                    false
                                                }
                                            }
                                            _ => unimplemented!(),
                                        }
                                    } else {
                                        panic!()
                                    }
                                })
                                .collect::<Vec<_>>();

                            if matches.is_empty() {
                                None
                            } else {
                                Some((reagent, matches))
                            }
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                log::trace!("Creates: {:?}", def.product);

                // Was this conducted at a "thing", or just out in the wild?
                if action.subjects.is_none() {
                    match &def.product.kind {
                        Kind::Item(item_name) => {
                            let product = def.product.clone();
                            let source_transform = transform_storage
                                .get(action.source.unwrap())
                                .unwrap()
                                .clone();

                            let lol = item_name.clone();

                            // fetch item material
                            // TODO: We need to make sure materials are uniform somewhere if its a crafting action
                            // If its just a 1-source consumption, copy its material

                            lazy.exec_mut(move |lazy_world| {
                                for _ in 0..product.count {
                                    crate::initializers::spawn_item_world(
                                        &lol,
                                        Some(source_transform.clone()),
                                        None,
                                        None,
                                        None,
                                        lazy_world,
                                    );
                                }
                            });
                        }
                        _ => unimplemented!(),
                    }
                }

                log::trace!("entities to consume {:?}", reagent_entities_map);
                reagent_entities_map.iter().for_each(|(_, matches)| {
                    matches.iter().for_each(|e| {
                        if let ActionTarget::Entity(e) = e {
                            self.delete.add(e.id());
                        }
                    })
                });

                // Find the subjects of this action which coorrespond to these reagents...

                // We are done, set the reaction as successful
                // TODO: Timers!
                current_action_storage
                    .get_mut(action.source.unwrap())
                    .unwrap()
                    .status = Ok(ActionStatus::Success);
            }

            if !self.delete.is_empty() {
                let delete = (&self.delete, &entities)
                    .join()
                    .map(|(_, entity)| entity)
                    .collect::<Vec<_>>();
                log::error!("Queueing delete entities: {:?}", delete);

                lazy.exec_mut(move |world| {
                    delete.into_iter().for_each(|entity| {
                        if let Err(e) = world.delete_entity(entity) {
                            log::error!("Deleting entity failed: {:?}", e);
                        }
                    });
                });
            }
            self.delete.clear();
        }
    }
}

impl<'a, 'b> SystemDesc<'a, 'b, ExecuteRactionSystem> for ExecuteRactionSystem {
    fn build(self, world: &mut World) -> Self {
        <Self as System<'_>>::SystemData::setup(world);

        let reader = Some(Write::<EventChannel<ActionEvent>>::fetch(world).register_reader());

        Self {
            reader,
            ..Self::default()
        }
    }
}
