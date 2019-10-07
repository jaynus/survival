#![allow(
    clippy::similar_names,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::type_complexity
)]
use crate::components::{BuildingComponent, CurrentActionComponent, ItemComponent, PawnComponent};
use core::{
    amethyst::{
        core::{
            components::Transform,
            math::{Point3, Vector3},
            SystemBundle, SystemDesc,
        },
        ecs::{
            prelude::DispatcherBuilder, world::Builder, BitSet, Entities, Entity, Join, LazyUpdate,
            Read, ReadExpect, ReadStorage, System, SystemData, World, WorldExt, Write,
            WriteStorage,
        },
        input::InputEvent,
        renderer::{debug_drawing::DebugLinesComponent, palette::Srgba},
        shrev::{EventChannel, ReaderId},
        tiles::{iters::Region, Map, TileMap},
    },
    fsm::ActionTarget,
    hibitset::BitSetLike,
    input::{ActionBinding, FilteredInputEvent, InputState, InputStateFlags, SelectionData},
    tiles::region::RegionTile,
};

type SelectionEvent = SelectionData;

pub struct MouseSelectionSystem {
    cur_select_data: Option<(Point3<f32>, Entity)>,
    reader_id: ReaderId<FilteredInputEvent>,
}
impl<'s> System<'s> for MouseSelectionSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, LazyUpdate>,
        Write<'s, InputState>,
        ReadStorage<'s, TileMap<RegionTile>>,
        ReadStorage<'s, PawnComponent>,
        ReadStorage<'s, ItemComponent>,
        ReadStorage<'s, BuildingComponent>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, DebugLinesComponent>,
        Write<'s, EventChannel<FilteredInputEvent>>,
        Write<'s, EventChannel<SelectionEvent>>,
    );

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::too_many_lines
    )]
    fn run(
        &mut self,
        (
            entities_res,
            lazy,
            mut input_state,
            tilemap_storage,
            pawn_storage,
            item_storage,
            building_storage,
            transforms,
            mut debug_lines_storage,
            input_channel,
            mut selection_channel,
        ): Self::SystemData,
    ) {
        for event in input_channel.read(&mut self.reader_id) {
            if let Some(map) = (&tilemap_storage).join().next() {
                // We have a valid camera

                match event {
                    FilteredInputEvent::Free(InputEvent::ActionPressed(ActionBinding::Select)) => {
                        log::debug!("Selection started: state={:?}", input_state.current);
                        if input_state.current != InputStateFlags::Normal {
                            continue;
                        }

                        input_state.update_selection(None);
                        input_state.current = InputStateFlags::Selection;

                        // New selection started
                        let debug_lines_component = DebugLinesComponent::with_capacity(4);

                        let draw_entity = lazy
                            .create_entity(&entities_res)
                            .with(debug_lines_component)
                            .build();

                        self.cur_select_data =
                            Some((input_state.mouse_world_position, draw_entity));
                    }
                    FilteredInputEvent::Free(InputEvent::ActionReleased(ActionBinding::Select))
                    | FilteredInputEvent::Filtered(InputEvent::ActionReleased(
                        ActionBinding::Select,
                    )) => {
                        if input_state.current != InputStateFlags::Selection {
                            continue;
                        }
                        input_state.current = InputStateFlags::Normal;

                        if self.cur_select_data.is_some() {
                            // Selection ended, because the action went off and we had a start point
                            let start_world = self.cur_select_data.as_ref().unwrap().0;

                            // Done, clear it
                            let entity = self.cur_select_data.unwrap().1;
                            lazy.exec_mut(move |world| {
                                world.delete_entity(entity).unwrap();
                            });
                            self.cur_select_data = None;

                            let end_world = input_state.mouse_world_position;

                            let (top, bottom) = {
                                let start_tile = map.to_tile(&start_world.coords);
                                let end_tile = map.to_tile(&end_world.coords);

                                if start_tile.is_none() {
                                    log::error!(
                                        "ActionReleased::Select: Invalid world-> tile conversion: {:?}",
                                        start_world.coords
                                    );
                                    return;
                                }
                                if end_tile.is_none() {
                                    log::error!(
                                        "ActionReleased::Select: Invalid world-> tile conversion: {:?}",
                                        end_world.coords
                                    );
                                    return;
                                }

                                let start_tile = start_tile.unwrap();
                                let end_tile = end_tile.unwrap();

                                let mut min = Point3::new(0, 0, 0);
                                let mut max = Point3::new(0, 0, 0);

                                if start_tile.x < end_tile.x {
                                    min.x = start_tile.x;
                                    max.x = end_tile.x + 1;
                                } else {
                                    min.x = end_tile.x;
                                    max.x = start_tile.x + 1;
                                };
                                if start_tile.y < end_tile.y {
                                    min.y = start_tile.y;
                                    max.y = end_tile.y + 1;
                                } else {
                                    min.y = end_tile.y;
                                    max.y = start_tile.y + 1;
                                };

                                (min, max)
                            };

                            let region = Region::new(top, bottom);
                            log::trace!("region = {:?}", region);
                            log::trace!("(top,bottom) = ({:?}, {:?})", top, bottom);
                            log::trace!("area = {:?}", region.volume());

                            if region.volume() > 0 {
                                let mut entities = BitSet::new();
                                // Find pawn entities within the selection

                                for (entity, transform, _) in
                                    (&entities_res, &transforms, &pawn_storage).join()
                                {
                                    if let Some(tile_coord) = map.to_tile(transform.translation()) {
                                        if region.contains(&tile_coord) {
                                            entities.add(entity.id());
                                        }
                                    }
                                }

                                if entities.is_empty() {
                                    // Find item entities within the selection
                                    for (entity, transform, _) in
                                        (&entities_res, &transforms, &item_storage).join()
                                    {
                                        if let Some(tile_coord) =
                                            map.to_tile(transform.translation())
                                        {
                                            if region.contains(&tile_coord) {
                                                entities.add(entity.id());
                                            }
                                        }
                                    }

                                    if entities.is_empty() {
                                        // Try buildings
                                        for (entity, transform, _) in
                                            (&entities_res, &transforms, &building_storage).join()
                                        {
                                            if let Some(tile_coord) =
                                                map.to_tile(transform.translation())
                                            {
                                                if region.contains(&tile_coord) {
                                                    let data = SelectionData::Building(entity);
                                                    input_state
                                                        .update_selection(Some(data.clone()));
                                                    selection_channel.single_write(data);

                                                    break;
                                                }
                                            }
                                        }
                                    } else {
                                        let data = SelectionData::ItemGroup {
                                            region: Some(region),
                                            entities: entities.clone(),
                                        };
                                        input_state.update_selection(Some(data.clone()));
                                        selection_channel.single_write(data);
                                    }
                                } else {
                                    let data = SelectionData::PawnGroup {
                                        region: Some(region),
                                        entities: entities.clone(),
                                    };
                                    input_state.update_selection(Some(data.clone()));
                                    selection_channel.single_write(data);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if self.cur_select_data.is_some() {
            // We have a valid camera
            let end = input_state.mouse_world_position;

            if let Some(component) = debug_lines_storage.get_mut(self.cur_select_data.unwrap().1) {
                component.clear();

                component.add_box(
                    self.cur_select_data.as_ref().unwrap().0,
                    end,
                    Srgba::new(0.5, 0.05, 0.65, 1.0),
                )
            }
        }
    }
}

#[derive(Default)]
pub struct MouseSelectionSystemDesc;
impl<'a, 'b> SystemDesc<'a, 'b, MouseSelectionSystem> for MouseSelectionSystemDesc {
    fn build(self, world: &mut World) -> MouseSelectionSystem {
        <MouseSelectionSystem as System<'_>>::SystemData::setup(world);

        let reader_id = Write::<EventChannel<FilteredInputEvent>>::fetch(world).register_reader();
        MouseSelectionSystem {
            cur_select_data: None,
            reader_id,
        }
    }
}

pub struct DrawSelectionBoxesSystem {
    selection_event_reader: ReaderId<SelectionEvent>,
    active_draw_entities: BitSet,
}
impl<'s> System<'s> for DrawSelectionBoxesSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, InputState>,
        Read<'s, EventChannel<SelectionEvent>>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, TileMap<RegionTile>>,
        WriteStorage<'s, DebugLinesComponent>,
    );

    fn run(
        &mut self,
        (
            entities_res,
            input_state,
            selection_channel,
            transform_storage,
            tilemap_storage,
            mut lines_storage,
        ): Self::SystemData,
    ) {
        for event in selection_channel.read(&mut self.selection_event_reader) {
            // Remove old debug lines
            (&entities_res, &self.active_draw_entities)
                .join()
                .for_each(|(entity, _)| {
                    lines_storage.remove(entity);
                });

            match event {
                SelectionData::ItemGroup { entities, .. }
                | SelectionData::PawnGroup { entities, .. } => {
                    (&entities_res, entities, &transform_storage)
                        .join()
                        .for_each(|(entity, _, _)| {
                            lines_storage
                                .insert(entity, DebugLinesComponent::with_capacity(4))
                                .unwrap();
                        });

                    self.active_draw_entities = entities.clone();
                }
                SelectionData::Building(entity) => {
                    lines_storage
                        .insert(*entity, DebugLinesComponent::with_capacity(4))
                        .unwrap();
                    self.active_draw_entities.add(entity.id());
                }
            }

            // save entities, add new debug lines
        }
        if let Some(map) = (&tilemap_storage).join().next() {
            if input_state.selection.is_none() && !self.active_draw_entities.is_empty() {
                (&entities_res, &self.active_draw_entities)
                    .join()
                    .for_each(|(entity, _)| {
                        lines_storage.remove(entity);
                    });
                self.active_draw_entities.clear();
            } else if !self.active_draw_entities.is_empty() {
                (
                    &self.active_draw_entities,
                    &transform_storage,
                    &mut lines_storage,
                )
                    .join()
                    .for_each(|(_, transform, lines)| {
                        lines.clear();

                        let tile_dimensions = Vector3::new(
                            map.tile_dimensions().x as f32,
                            map.tile_dimensions().y as f32,
                            map.tile_dimensions().z as f32,
                        );

                        let min = transform.translation() - (tile_dimensions / 2.0)
                            + Vector3::new(0.0, 0.0, 1.0);

                        let max = transform.translation()
                            + (tile_dimensions / 2.0)
                            + Vector3::new(0.0, 0.0, 1.0);

                        lines.add_box(
                            Point3::from(min),
                            Point3::from(max),
                            Srgba::new(0.5, 0.05, 0.65, 1.0),
                        );
                    });
            }
        }
    }
}

#[derive(Default)]
pub struct DrawSelectionBoxesSystemDesc;
impl<'a, 'b> SystemDesc<'a, 'b, DrawSelectionBoxesSystem> for DrawSelectionBoxesSystemDesc {
    fn build(self, world: &mut World) -> DrawSelectionBoxesSystem {
        <DrawSelectionBoxesSystem as System<'_>>::SystemData::setup(world);

        let selection_event_reader =
            Write::<EventChannel<SelectionEvent>>::fetch(world).register_reader();
        DrawSelectionBoxesSystem {
            selection_event_reader,
            active_draw_entities: BitSet::default(),
        }
    }
}

pub struct UserMovePawnSystem {
    reader_id: ReaderId<FilteredInputEvent>,
}
impl<'s> System<'s> for UserMovePawnSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, LazyUpdate>,
        Read<'s, InputState>,
        ReadStorage<'s, TileMap<RegionTile>>,
        WriteStorage<'s, DebugLinesComponent>,
        WriteStorage<'s, CurrentActionComponent>,
        Read<'s, EventChannel<FilteredInputEvent>>,
    );

    fn run(
        &mut self,
        (
            entities,
            _lazy,
            input_state,
            tilemap_storage,
            _debug_lines_storage,
            mut current_action_storage,
            input_channel,
        ): Self::SystemData,
    ) {
        for event in input_channel.read(&mut self.reader_id) {
            let map = if let Some(tile_map) = (&tilemap_storage).join().next() {
                tile_map
            } else {
                continue;
            };

            if input_state.current != InputStateFlags::Normal || input_state.selection.is_none() {
                continue;
            }

            let pawns = {
                match input_state.selection.as_ref().unwrap() {
                    SelectionData::PawnGroup { entities, .. } => entities,
                    _ => continue,
                }
            };

            if let FilteredInputEvent::Free(InputEvent::ActionReleased(ActionBinding::DoAction)) =
                event
            {
                log::trace!("Move action released");
                // Release of a press action
                // We know selection is some, and we have a mouse release
                let world_coords = input_state.mouse_world_position.coords;
                log::trace!("world_coords = {:?}", world_coords);
                let target_tile_pos = if let Some(pos) = map.to_tile(&world_coords) {
                    pos
                } else {
                    log::error!(
                        "MovePawn: Invalid world-> tile conversion: {:?}",
                        world_coords
                    );
                    return;
                };

                log::trace!("move target = {:?}", target_tile_pos);
                // TODO:
                log::error!("Disabled for behavior change");
            }
        }
    }
}

#[derive(Default)]
pub struct UserMovePawnSystemDesc;
impl<'a, 'b> SystemDesc<'a, 'b, UserMovePawnSystem> for UserMovePawnSystemDesc {
    fn build(self, world: &mut World) -> UserMovePawnSystem {
        <UserMovePawnSystem as System<'_>>::SystemData::setup(world);

        let reader_id = Write::<EventChannel<FilteredInputEvent>>::fetch(world).register_reader();
        UserMovePawnSystem { reader_id }
    }
}

#[derive(Default)]
pub struct InputSelectionBundle;
impl<'a, 'b> SystemBundle<'a, 'b> for InputSelectionBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), core::amethyst::Error> {
        builder.add(
            MouseSelectionSystemDesc::default().build(world),
            "MouseSelectionSystem",
            &["FilterInputSystem", "RegionCameraMovementSystem"],
        );
        builder.add(
            DrawSelectionBoxesSystemDesc::default().build(world),
            "DrawSelectionBoxesSystem",
            &[
                "FilterInputSystem",
                "MouseSelectionSystem",
                "RegionCameraMovementSystem",
            ],
        );
        builder.add(
            UserMovePawnSystemDesc::default().build(world),
            "UserMovePawnSystem",
            &["FilterInputSystem", "RegionCameraMovementSystem"],
        );

        Ok(())
    }
}
