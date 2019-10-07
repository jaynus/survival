use core::{
    amethyst::{
        core::{
            components::Transform,
            ecs::{
                Builder, DispatcherBuilder, Entities, Entity, Join, LazyUpdate, Read, ReadExpect,
                ReadStorage, System, SystemData, World, Write, WriteExpect, WriteStorage,
            },
            geometry::Plane,
            math::{Point2, Point3, Vector2, Vector3},
            SystemBundle, SystemDesc,
        },
        input::{InputEvent, InputHandler},
        renderer::{debug_drawing::DebugLinesComponent, palette::Srgba, ActiveCamera, Camera},
        shrev::{EventChannel, ReaderId},
        tiles::{iters::Region, Map, TileMap},
        window::ScreenDimensions,
        winit::VirtualKeyCode,
    },
    embark::EmbarkSettings,
    input::{AxisBinding, FilteredInputEvent, InputState},
    tiles::world::WorldTile,
};

pub struct WorldCameraMovementSystem {
    reader_id: ReaderId<FilteredInputEvent>,
}
impl<'s> System<'s> for WorldCameraMovementSystem {
    type SystemData = (
        Read<'s, ActiveCamera>,
        ReadExpect<'s, ScreenDimensions>,
        Entities<'s>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, TileMap<WorldTile>>,
        WriteStorage<'s, Transform>,
        Read<'s, EventChannel<FilteredInputEvent>>,
        Write<'s, InputState>,
        Read<'s, InputHandler<core::input::BindingTypes>>,
    );

    #[allow(clippy::cast_precision_loss)]
    fn run(
        &mut self,
        (
            active_camera,
            screen_dimensions,
            entities,
            cameras,
            tile_maps,
            mut transforms,
            filtered_input_channel,
            mut input_state,
            input,
        ): Self::SystemData,
    ) {
        for event in filtered_input_channel.read(&mut self.reader_id) {
            match event {
                FilteredInputEvent::Free(InputEvent::MouseMoved { .. })
                | FilteredInputEvent::Filtered(InputEvent::MouseMoved { .. }) => {
                    let mut camera_join = (&cameras, &mut transforms).join();
                    if let Some((camera, camera_transform)) = active_camera
                        .entity
                        .and_then(|a| camera_join.get(a, &entities))
                        .or_else(|| camera_join.next())
                    {
                        if let Some(mouse_pos) = input.mouse_position() {
                            let ray = camera.projection().screen_ray(
                                Point2::new(mouse_pos.0, mouse_pos.1),
                                Vector2::new(screen_dimensions.width(), screen_dimensions.height()),
                                camera_transform,
                            );
                            if let Some(distance) = ray.intersect_plane(&Plane::with_z(0.0)) {
                                input_state.mouse_world_position = ray.at_distance(distance);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(map) = (&tile_maps).join().next() {
            let accel = if input.key_is_down(VirtualKeyCode::LShift) {
                10.0
            } else {
                1.0
            };

            let x_move = input.axis_value(&AxisBinding::CameraX).unwrap() * accel;
            let y_move = input.axis_value(&AxisBinding::CameraY).unwrap() * accel;
            let z_move = input.axis_value(&AxisBinding::CameraZ).unwrap() * accel;

            let mut camera_join = (&cameras, &mut transforms).join();
            if let Some((_, camera_transform)) = active_camera
                .entity
                .and_then(|a| camera_join.get(a, &entities))
                .or_else(|| camera_join.next())
            {
                let scale = camera_transform.scale().x;

                if x_move != 0.0 || y_move != 0.0 {
                    log::trace!("z_move = {}", z_move);
                    let mut translation = *camera_transform.translation();
                    translation.x += x_move * 5.0 * scale;
                    translation.y += y_move * 5.0 * scale;

                    let zoom = translation.z - translation.z.floor();
                    log::trace!("Zoom = {}", zoom);
                    let camera_z = 0.0 * map.tile_dimensions().z as f32;
                    translation.z = camera_z + zoom;
                    log::trace!("camera_z: {}", camera_z);
                    log::trace!("New translation: {}", translation);

                    camera_transform.set_translation(translation);
                }

                if z_move != 0.0 {
                    let mut z_scale = scale + (z_move * 0.2);
                    println!("z_scale = {}", z_scale);

                    if z_scale < 0.4 {
                        z_scale = 0.4;
                    }

                    z_scale = (z_scale * 100.0).round() / 100.0;

                    let scale = Vector3::new(z_scale, z_scale, z_scale);
                    println!("Scale = {:?}", scale);

                    camera_transform.set_scale(scale);
                }
            }
        }
    }
}

#[derive(Default)]
pub struct WorldCameraMovementSystemDesc;
impl<'a, 'b> SystemDesc<'a, 'b, WorldCameraMovementSystem> for WorldCameraMovementSystemDesc {
    fn build(self, world: &mut World) -> WorldCameraMovementSystem {
        <WorldCameraMovementSystem as System<'_>>::SystemData::setup(world);

        let reader_id = Write::<EventChannel<FilteredInputEvent>>::fetch(world).register_reader();

        WorldCameraMovementSystem { reader_id }
    }
}

pub struct EmbarkSelectionSystem {
    reader_id: ReaderId<FilteredInputEvent>,
    draw_entity: Option<Entity>,
}
impl<'s> System<'s> for EmbarkSelectionSystem {
    type SystemData = (
        Read<'s, LazyUpdate>,
        Entities<'s>,
        ReadStorage<'s, TileMap<WorldTile>>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, DebugLinesComponent>,
        WriteExpect<'s, EmbarkSettings>,
        Read<'s, EventChannel<FilteredInputEvent>>,
        Read<'s, InputState>,
    );

    #[allow(clippy::cast_precision_loss)]
    fn run(
        &mut self,
        (
            lazy,
            entities_res,
            tilemap_storage,
            _,
            mut debug_lines_storage,
            mut embark_settings,
            filtered_input_channel,
            input_state,
        ): Self::SystemData,
    ) {
        for _ in filtered_input_channel.read(&mut self.reader_id) {}
        if let Some(draw_entity) = self.draw_entity {
            if let Some(lines) = debug_lines_storage.get_mut(draw_entity) {
                lines.clear();
                if let Some(map) = (&tilemap_storage).join().next() {
                    if let Some(tile_pos) = map.to_tile(&input_state.mouse_world_position.coords) {
                        let half_d = map.tile_dimensions().x as f32 / 2.0;
                        let half = Vector3::new(half_d, -half_d, 0.0);

                        let start = Point3::from(map.to_world(&tile_pos) - half);

                        let end_tile = tile_pos
                            + Vector3::new(
                                embark_settings.world_dimensions.x - 1,
                                embark_settings.world_dimensions.y - 1,
                                0,
                            );
                        let end = Point3::from(map.to_world(&end_tile) + half);

                        lines.add_box(start, end, Srgba::new(0.5, 0.05, 0.65, 1.0));

                        // Set the current embark coordinate
                        embark_settings.region = Some(Region::new(tile_pos, end_tile));
                    }
                }
            }
        } else {
            // We havnt created a draw entity yet, do it
            self.draw_entity = Some(
                lazy.create_entity(&entities_res)
                    .with(Transform::default())
                    .with(DebugLinesComponent::with_capacity(4))
                    .build(),
            );
        }
    }
}

#[derive(Default)]
pub struct EmbarkSelectionSystemDesc;
impl<'a, 'b> SystemDesc<'a, 'b, EmbarkSelectionSystem> for EmbarkSelectionSystemDesc {
    fn build(self, world: &mut World) -> EmbarkSelectionSystem {
        <EmbarkSelectionSystem as System<'_>>::SystemData::setup(world);

        let reader_id = Write::<EventChannel<FilteredInputEvent>>::fetch(world).register_reader();

        EmbarkSelectionSystem {
            reader_id,
            draw_entity: None,
        }
    }
}

#[derive(Default)]
pub struct WorldViewBundle;
impl<'a, 'b> SystemBundle<'a, 'b> for WorldViewBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), core::amethyst::Error> {
        builder.add(
            WorldCameraMovementSystemDesc::default().build(world),
            "WorldCameraMovementSystem",
            &["FilterInputSystem"],
        );

        builder.add(
            EmbarkSelectionSystemDesc::default().build(world),
            "EmbarkSelectionSystemDesc",
            &["FilterInputSystem"],
        );

        Ok(())
    }
}
