use core::{
    amethyst::{
        core::{SystemDesc, Transform},
        ecs::{
            Builder, Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System, SystemData,
            World, WorldExt, Write, WriteStorage,
        },
        input::InputEvent,
        renderer::{palette::Srgba, resources::Tint},
        shrev::{EventChannel, ReaderId},
        tiles::{Map, TileMap},
    },
    defs::{building::BuildingDefinition, sprites::SpriteOntoFlags, DefinitionStorage, Named},
    hibitset::{BitSet, BitSetLike},
    input::{ActionBinding, FilteredInputEvent, InputState, InputStateFlags, PlayerInputEvent},
    settings::GraphicsSettings,
    tiles::region::RegionTile,
};

pub struct DrawPlacementEntitySystem {
    filtered_input_reader: ReaderId<FilteredInputEvent>,
    player_input_reader: ReaderId<PlayerInputEvent>,
    active_draw_entities: BitSet,
    cur_building_id: Option<u32>,
}
impl<'s> System<'s> for DrawPlacementEntitySystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, LazyUpdate>,
        Write<'s, InputState>,
        ReadExpect<'s, DefinitionStorage<BuildingDefinition>>,
        ReadExpect<'s, GraphicsSettings>,
        Read<'s, EventChannel<FilteredInputEvent>>,
        Read<'s, EventChannel<PlayerInputEvent>>,
        ReadStorage<'s, TileMap<RegionTile>>,
        WriteStorage<'s, Transform>,
    );

    #[allow(unreachable_patterns)]
    fn run(
        &mut self,
        (
            entities,
            lazy,
            mut input_state,
            building_defs,
            config,
            filtered_input_channel,
            player_input_channel,
            maps_storage,
            mut transform_storage,
        ): Self::SystemData,
    ) {
        let map = if let Some(map) = (&maps_storage).join().next() {
            map
        } else {
            return;
        };

        for event in player_input_channel.read(&mut self.player_input_reader) {
            match event {
                PlayerInputEvent::StartBuildingPlacement { building_id } => {
                    self.cur_building_id = Some(*building_id);
                    let building = building_defs.get(*building_id).expect("Invalid building");

                    let transform = Transform::default();

                    let mut builder = lazy
                        .create_entity(&entities)
                        .with(transform)
                        .with(Tint(Srgba::new(0.3, 1.0, 0.3, 0.7)));
                    builder = building.sprite.onto_builder(
                        builder,
                        core::z_level_modifiers::TOP,
                        SpriteOntoFlags::SkipTint,
                        &config,
                    );

                    let entity = builder.build();

                    self.active_draw_entities.add(entity.id());

                    input_state.current = InputStateFlags::Placement;
                    input_state.update_selection(None);
                }
                _ => continue,
            }
        }
        let mouse_world = input_state.mouse_world_position;

        if !self.active_draw_entities.is_empty() {
            // Get the current mouse world position

            // To snap, we convert to-tile and back.
            if let Some(tile_pos) = map.to_tile(&mouse_world.coords) {
                let final_pos = map.to_world(&tile_pos);

                (&entities, &self.active_draw_entities)
                    .join()
                    .for_each(|(e, _)| {
                        if let Some(transform) = transform_storage.get_mut(e) {
                            transform.set_translation(final_pos);
                        }
                    });
            }
        }

        for event in filtered_input_channel.read(&mut self.filtered_input_reader) {
            match event {
                FilteredInputEvent::Filtered(InputEvent::ActionPressed(
                    ActionBinding::DoAction,
                ))
                | FilteredInputEvent::Free(InputEvent::ActionPressed(ActionBinding::DoAction)) => {
                    // Cancel action
                    input_state.current = InputStateFlags::Normal;
                    self.cur_building_id = None;
                }
                FilteredInputEvent::Free(InputEvent::ActionReleased(ActionBinding::Select)) => {
                    if input_state.current == InputStateFlags::Placement
                        && self.cur_building_id.is_some()
                    {
                        // Perform the actual placement!
                        // TODO: Construction timing system!
                        log::debug!(
                            "Performing building construction placement!: {:?}",
                            self.cur_building_id
                        );

                        if let Some(tile_pos) = map.to_tile(&mouse_world.coords) {
                            let building_name = building_defs
                                .get(self.cur_building_id.unwrap())
                                .unwrap()
                                .name()
                                .to_string();

                            //crate::initializers::spawn_building();
                            lazy.exec_mut(move |world| {
                                crate::initializers::spawn_building(
                                    &building_name,
                                    &tile_pos,
                                    world,
                                );
                            });

                            input_state.current = InputStateFlags::Normal;
                            self.cur_building_id = None;
                        }
                    }
                }
                _ => {}
            }
        }

        if !self.active_draw_entities.is_empty()
            && input_state.current != InputStateFlags::Placement
        {
            // We arn't in placement state anymore, just clear ourselves
            let entities = (&entities, &self.active_draw_entities)
                .join()
                .map(|(e, _)| e)
                .collect::<Vec<_>>();
            self.active_draw_entities.clear();
            lazy.exec_mut(move |world| {
                entities
                    .iter()
                    .for_each(|e| world.delete_entity(*e).unwrap());
            });
        }
    }
}

#[derive(Default)]
pub struct DrawPlacementEntitySystemDesc;
impl<'a, 'b> SystemDesc<'a, 'b, DrawPlacementEntitySystem> for DrawPlacementEntitySystemDesc {
    fn build(self, world: &mut World) -> DrawPlacementEntitySystem {
        <DrawPlacementEntitySystem as System<'_>>::SystemData::setup(world);

        let filtered_input_reader =
            Write::<EventChannel<FilteredInputEvent>>::fetch(world).register_reader();
        let player_input_reader =
            Write::<EventChannel<PlayerInputEvent>>::fetch(world).register_reader();
        DrawPlacementEntitySystem {
            filtered_input_reader,
            player_input_reader,
            active_draw_entities: BitSet::default(),
            cur_building_id: None,
        }
    }
}
