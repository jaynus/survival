pub mod selection;
pub use selection::{DrawSelectionBoxesSystem, InputSelectionBundle, MouseSelectionSystem};

pub mod placement;
use placement::DrawPlacementEntitySystemDesc;

use amethyst_imgui::ImguiContextWrapper;
use core::input::BindingTypes as SurvivalBindingTypes;
use core::{
    amethyst::{
        core::{
            components::Transform,
            ecs::{
                DispatcherBuilder, Entities, Join, Read, ReadExpect, ReadStorage, System,
                SystemData, World, Write, WriteStorage,
            },
            geometry::Plane,
            math::{Point2, Vector2, Vector3},
            SystemBundle, SystemDesc,
        },
        input::{InputEvent, InputHandler},
        renderer::{ActiveCamera, Camera},
        shrev::{EventChannel, ReaderId},
        tiles::{Map, TileMap},
        window::ScreenDimensions,
        winit::{Event, VirtualKeyCode},
    },
    input::{ActionBinding, AxisBinding, FilteredInputEvent, InputState},
    tiles::{region::RegionTile, CurrentTileZ},
};
use std::sync::{Arc, Mutex};

pub struct FilterInputSystem {
    input_reader: ReaderId<InputEvent<SurvivalBindingTypes>>,
    winit_reader: ReaderId<Event>,
}
impl<'s> System<'s> for FilterInputSystem {
    type SystemData = (
        ReadExpect<'s, Arc<Mutex<ImguiContextWrapper>>>,
        Read<'s, EventChannel<InputEvent<SurvivalBindingTypes>>>,
        Read<'s, EventChannel<Event>>,
        Write<'s, EventChannel<FilteredInputEvent>>,
    );

    fn run(
        &mut self,
        (context, input_events, winit_events, mut filtered_events): Self::SystemData,
    ) {
        let state = &mut context.lock().unwrap().0;

        for _ in winit_events.read(&mut self.winit_reader) {}
        for input in input_events.read(&mut self.input_reader) {
            let mut taken = false;
            match input {
                InputEvent::MouseMoved { .. }
                | InputEvent::MouseButtonPressed(_)
                | InputEvent::MouseButtonReleased(_)
                | InputEvent::MouseWheelMoved(_) => {
                    if state.io().want_capture_mouse {
                        taken = true;
                    }
                }
                InputEvent::KeyPressed { .. } | InputEvent::KeyReleased { .. } => {
                    if state.io().want_capture_keyboard {
                        taken = true;
                    }
                }
                InputEvent::ActionPressed(action) => match action {
                    _ => {
                        if state.io().want_capture_mouse {
                            taken = true;
                        }
                    }
                },
                InputEvent::ActionReleased(action) => match action {
                    _ => {
                        if state.io().want_capture_mouse || state.io().want_capture_keyboard {
                            taken = true;
                        }
                    }
                },
                _ => {}
            }

            if taken {
                filtered_events.single_write(FilteredInputEvent::Filtered(input.clone()));
            } else {
                filtered_events.single_write(FilteredInputEvent::Free(input.clone()));
            }
        }
    }
}

#[derive(Default)]
pub struct FilterInputSystemDesc;
impl<'a, 'b> SystemDesc<'a, 'b, FilterInputSystem> for FilterInputSystemDesc {
    fn build(self, world: &mut World) -> FilterInputSystem {
        <FilterInputSystem as System<'_>>::SystemData::setup(world);

        let input_reader =
            Write::<EventChannel<InputEvent<SurvivalBindingTypes>>>::fetch(world).register_reader();
        let winit_reader = Write::<EventChannel<Event>>::fetch(world).register_reader();

        FilterInputSystem {
            input_reader,
            winit_reader,
        }
    }
}

pub struct RegionCameraMovementSystem {
    reader_id: ReaderId<FilteredInputEvent>,
}
impl<'s> System<'s> for RegionCameraMovementSystem {
    type SystemData = (
        Read<'s, ActiveCamera>,
        ReadExpect<'s, ScreenDimensions>,
        Entities<'s>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, TileMap<RegionTile>>,
        WriteStorage<'s, Transform>,
        Read<'s, EventChannel<FilteredInputEvent>>,
        Write<'s, InputState>,
        Read<'s, InputHandler<core::input::BindingTypes>>,
        Write<'s, CurrentTileZ>,
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
            mut current_tile_z,
        ): Self::SystemData,
    ) {
        let mut cur_tile_changed = false;

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
                            if let Some(distance) =
                                ray.intersect_plane(&Plane::with_z(current_tile_z.0 as f32))
                            {
                                input_state.mouse_world_position = ray.at_distance(distance);
                            }
                        }
                    }
                }

                FilteredInputEvent::Free(InputEvent::ActionPressed(ActionBinding::UpZ))
                | FilteredInputEvent::Filtered(InputEvent::ActionPressed(ActionBinding::UpZ)) => {
                    if let Some(map) = (&tile_maps).join().next() {
                        if current_tile_z.0 != 0 {
                            current_tile_z.0 -= 1;

                            cur_tile_changed = true;

                            (current_tile_z.1).0 =
                                (map.tile_dimensions().z * current_tile_z.0) as f32;
                            (current_tile_z.1).1 =
                                ((map.tile_dimensions().z + 1) * current_tile_z.0) as f32 - 0.0001;

                            log::trace!("Set current z: {}", current_tile_z.0);
                        }
                    }
                }
                FilteredInputEvent::Free(InputEvent::ActionPressed(ActionBinding::DownZ))
                | FilteredInputEvent::Filtered(InputEvent::ActionPressed(ActionBinding::DownZ)) => {
                    if let Some(map) = (&tile_maps).join().next() {
                        if current_tile_z.0 < map.dimensions().z {
                            current_tile_z.0 += 1;
                            cur_tile_changed = true;

                            (current_tile_z.1).0 =
                                (map.tile_dimensions().z * current_tile_z.0) as f32;
                            (current_tile_z.1).1 =
                                ((map.tile_dimensions().z + 1) * current_tile_z.0) as f32 - 0.0001;

                            log::trace!("Set current z: {}", current_tile_z.0);
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
                let scale_accel = camera_transform.scale().x;

                if x_move != 0.0 || y_move != 0.0 || cur_tile_changed {
                    log::trace!("z_move = {}", z_move);
                    let mut translation = *camera_transform.translation();
                    translation.x += x_move * 5.0 * scale_accel;
                    translation.y += y_move * 5.0 * scale_accel;

                    let zoom = translation.z - translation.z.floor();
                    log::trace!("Zoom = {}", zoom);
                    let camera_z = current_tile_z.0 as f32 * map.tile_dimensions().z as f32;
                    translation.z = camera_z + zoom;
                    log::trace!("camera_z: {}", camera_z);
                    log::trace!("New translation: {}", translation);

                    camera_transform.set_translation(translation);
                }

                if z_move != 0.0 {
                    let z_scale = 0.1 * z_move;
                    let scale = camera_transform.scale();
                    let scale =
                        Vector3::new(scale.x + z_scale, scale.y + z_scale, scale.z + z_scale);
                    camera_transform.set_scale(scale);
                }
            }
        }
    }
}

#[derive(Default)]
pub struct RegionCameraMovementSystemDesc;
impl<'a, 'b> SystemDesc<'a, 'b, RegionCameraMovementSystem> for RegionCameraMovementSystemDesc {
    fn build(self, world: &mut World) -> RegionCameraMovementSystem {
        <RegionCameraMovementSystem as System<'_>>::SystemData::setup(world);

        let reader_id = Write::<EventChannel<FilteredInputEvent>>::fetch(world).register_reader();

        RegionCameraMovementSystem { reader_id }
    }
}

#[derive(Default)]
pub struct InputBundle;
impl<'a, 'b> SystemBundle<'a, 'b> for InputBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), core::amethyst::Error> {
        builder.add(
            FilterInputSystemDesc::default().build(world),
            "FilterInputSystem",
            &["input_system"],
        );

        builder.add(
            RegionCameraMovementSystemDesc::default().build(world),
            "RegionCameraMovementSystem",
            &["input_system"],
        );

        builder.add(
            DrawPlacementEntitySystemDesc::default().build(world),
            "DrawPlacementEntitySystem",
            &["FilterInputSystem"],
        );

        selection::InputSelectionBundle::default().build(world, builder)
    }
}
