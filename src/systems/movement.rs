use crate::components::{CurrentActionComponent, CurrentPathingComponent, PropertiesComponent};
use ai::pathing::{PathingRequestEvent, PathingResponseEvent};
use core::{
    amethyst::{
        core::{
            components::Transform,
            math::{distance, Point3},
            SystemDesc, Time,
        },
        ecs::{
            storage::{GenericReadStorage, GenericWriteStorage},
            Component, Entities, Entity, Join, Read, ReadStorage, System, SystemData, VecStorage,
            World, Write, WriteStorage,
        },
        shrev::{EventChannel, ReaderId},
        tiles::{Map, TileMap},
    },
    defs::property::{Property, PropertyKind},
    fsm::{ActionEvent, ActionStatus, ActionTarget, Event, MovementEvent},
    tiles::region::RegionTile,
};

#[derive(Default)]
pub struct MovementTrackComponent {
    current_path_index: Option<usize>,
}
impl Component for MovementTrackComponent {
    type Storage = VecStorage<Self>;
}

pub struct PathingMovementSystem {
    action_reader: ReaderId<ActionEvent>,
    pathing_reader: ReaderId<PathingResponseEvent>,
    done: Vec<(ActionStatus, Entity)>,
}
impl<'s> System<'s> for PathingMovementSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, Time>,
        Read<'s, EventChannel<ActionEvent>>,
        Write<'s, EventChannel<PathingRequestEvent>>,
        Read<'s, EventChannel<PathingResponseEvent>>,
        ReadStorage<'s, TileMap<RegionTile>>,
        ReadStorage<'s, PropertiesComponent>,
        ReadStorage<'s, CurrentPathingComponent>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, CurrentActionComponent>,
        WriteStorage<'s, MovementTrackComponent>,
    );

    #[allow(clippy::too_many_lines)]
    fn run(
        &mut self,
        (
            entities,
            time,
            action_channel,
            mut path_request_channel,
            path_response_channel,
            tilemap_storage,
            property_storage,
            current_pathing_storage,
            mut transform_storage,
            mut active_action_storage,
            mut track_storage,
        ): Self::SystemData,
    ) {
        self.done.clear();

        let map = if let Some(tile_map) = (&tilemap_storage).join().next() {
            tile_map
        } else {
            return;
        };

        //log::trace!("delta={}", time.delta_time().as_millis());
        for action in action_channel.read(&mut self.action_reader) {
            let movement = if let Event::Move(movement) = action.event {
                movement
            } else {
                continue;
            };

            // Verify the pawn isnt moving already first
            if track_storage.contains(action.source.unwrap()) {
                // panic!("Encountered a new movement action for an already moving pawn?");
                track_storage.remove(action.source.unwrap());
            }

            let target_tile = match action.targets.as_ref().unwrap()[0] {
                ActionTarget::Entity(e) => map
                    .to_tile(&transform_storage.get(e).unwrap().translation())
                    .unwrap(),
                ActionTarget::Location(t) => t,
            };

            self.new_move_action(
                action.source.unwrap(),
                target_tile,
                movement,
                map,
                &mut track_storage,
                &transform_storage,
                &property_storage,
                &mut path_request_channel,
            );
        }

        // TODO: If a pathing request failed, we need to handle it
        for response in path_response_channel.read(&mut self.pathing_reader) {
            match response {
                Ok((request, _)) => {
                    if let Some(track) = track_storage.get_mut(request.entity) {
                        track.current_path_index = Some(0);
                    } else {
                        log::warn!("Got a path response for an untracked entity?");
                    }
                }
                Err(e) => {
                    log::warn!("Pathing failed to destination, re-requesting a path...");
                    /*
                    let source_tile_pos = map
                        .to_tile(
                            transform_storage
                                .get(e.request.entity)
                                .unwrap()
                                .translation(),
                        )
                        .unwrap();
                    path_request_channel.single_write(PathingRequestEvent::new(
                        e.request.entity,
                        source_tile_pos,
                        e.request.destination,
                        e.request.kind,
                    ));
                    */
                    self.done.push((ActionStatus::Failure, e.request.entity));
                }
            }
        }

        // Remove track components from cancelled actions
        // An action is cancelled with its currentactive is removed
        (&entities, &track_storage, !&active_action_storage)
            .join()
            .for_each(|(entity, _, _)| {
                log::debug!("Movement canceled on entity: {:?}", entity);
                self.done.push((ActionStatus::Cancelled, entity))
            });

        self.done.iter().for_each(|(_, entity)| {
            log::debug!("Removing cancelled action");
            track_storage.remove(*entity);
        });

        self.done.clear();

        // Handle active pawns
        for (entity, track, path, active_action, props) in (
            &entities,
            &mut track_storage,
            &current_pathing_storage,
            &active_action_storage,
            &property_storage,
        )
            .join()
        {
            if track.current_path_index.is_none() {
                continue;
            }

            log::trace!("Handling pathed + tracked pawn");
            if let Some(path_result) = &path.current_path {
                if let Ok(path_result) = path_result {
                    let (request, path) = path_result;
                    log::trace!("Valid path, result and track");

                    // Lets make sure the current action entity hasn't moved from our pathing destination
                    // if we are targetting an entity
                    if Event::Move(MovementEvent::Target) == active_action.inner.event {
                        if let ActionTarget::Entity(target) =
                            active_action.inner.targets.as_ref().unwrap()[0]
                        {
                            if transform_storage.get(target).is_some() {
                                if map
                                    .to_tile(&transform_storage.get(target).unwrap().translation())
                                    .unwrap()
                                    != request.destination
                                {
                                    log::trace!("Target for pathing as moved, failing");
                                    self.done.push((ActionStatus::Failure, entity));
                                    continue;
                                }
                            } else {
                                log::trace!("Target disappeared!");
                                self.done.push((ActionStatus::Failure, entity));
                                continue;
                            }
                        }
                    }

                    let transform = transform_storage.get_mut(entity).unwrap();

                    // Are we currently on the tile?
                    let current_morton = map
                        .encode(&map.to_tile(transform.translation()).unwrap())
                        .unwrap();

                    let next_tile = map
                        .decode(path.path[track.current_path_index.unwrap()])
                        .unwrap();
                    let target_world_pos = map.to_world(&next_tile);

                    log::trace!("source = {:?}", transform.translation());
                    log::trace!("target = {:?}", target_world_pos);

                    // Move towards the next tile
                    if let Some(Property::MovementSpeed(movement_speed)) =
                        props.get(PropertyKind::MovementSpeed)
                    {
                        #[allow(clippy::cast_precision_loss)]
                        let move_factor = ((*movement_speed as f32) * 0.001) * time.delta_seconds();
                        let direction = (target_world_pos - transform.translation()).normalize();
                        let distance = distance(
                            &Point3::from(*transform.translation()),
                            &Point3::from(target_world_pos),
                        );
                        log::trace!("move_factor = {:?}:", move_factor);
                        log::trace!("direction = {:?}:", direction);
                        log::trace!("distance = {:?}:", distance);
                        let new_translation = if distance < move_factor {
                            target_world_pos
                        } else {
                            transform.translation() + (direction * move_factor)
                        };

                        transform.set_translation(new_translation);

                        if current_morton == path.path[track.current_path_index.unwrap()] {
                            *track.current_path_index.as_mut().unwrap() += 1;
                        }

                        if track.current_path_index.unwrap() >= path.path.len() {
                            // We are done!
                            self.done.push((ActionStatus::Success, entity));
                            continue;
                        }
                    }
                } else {
                    log::warn!("path result errored or is invalid: {:?}", path_result);
                }
            } else {
                log::warn!("current path is invalid");
            }
        }

        for (status, entity) in &self.done {
            log::trace!("Movement complete, clearing...");
            if let Some(active_action) = active_action_storage.get_mut(*entity) {
                active_action.status = Ok(*status);
            }
            //if let Some(path) = current_pathing_storage.get_mut(*entity) {
            //    path.complete();
            //}
            track_storage.remove(*entity);
        }
    }
}
impl PathingMovementSystem {
    fn new_move_action<TransformStorage, TrackStorage, PropertyStorage>(
        &mut self,
        entity: Entity,
        target: Point3<u32>,
        movement: MovementEvent,
        map: &dyn Map,
        track_storage: &mut TrackStorage,
        transform_storage: &TransformStorage,
        property_storage: &PropertyStorage,
        path_request_channel: &mut EventChannel<PathingRequestEvent>,
    ) where
        PropertyStorage: GenericReadStorage<Component = PropertiesComponent>,
        TransformStorage: GenericReadStorage<Component = Transform>,
        TrackStorage: GenericWriteStorage<Component = MovementTrackComponent>,
    {
        log::trace!("inserting tracking component (this could overwrite!)");
        track_storage
            .insert(entity, MovementTrackComponent::default())
            .unwrap();

        let props = property_storage.get(entity).unwrap();
        let movement_flag =
            if let Some(Property::Movement(movement_flag)) = props.get(PropertyKind::Movement) {
                movement_flag
            } else {
                panic!("Movement event for source with no movement properties")
            };

        // locations
        let source_tile_pos = map
            .to_tile(&transform_storage.get(entity).unwrap().translation())
            .unwrap();

        let dest_tile_pos = match movement {
            MovementEvent::To(point) => point,
            MovementEvent::Target => target,
            _ => unimplemented!("Unsupported movement type"),
        };

        // Lets immediately fire a path to target request
        path_request_channel.single_write(PathingRequestEvent::new(
            entity,
            source_tile_pos,
            dest_tile_pos,
            *movement_flag,
        ));

        log::trace!("Got a movement action! Lets start tracking the movement for htis entity");
    }
}

#[derive(Default)]
pub struct PathingMovementSystemDesc;
impl<'a, 'b> SystemDesc<'a, 'b, PathingMovementSystem> for PathingMovementSystemDesc {
    fn build(self, world: &mut World) -> PathingMovementSystem {
        <PathingMovementSystem as System<'_>>::SystemData::setup(world);

        let action_reader = Write::<EventChannel<ActionEvent>>::fetch(world).register_reader();
        let pathing_reader =
            Write::<EventChannel<PathingResponseEvent>>::fetch(world).register_reader();

        PathingMovementSystem {
            action_reader,
            pathing_reader,
            done: Vec::new(),
        }
    }
}
