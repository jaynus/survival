use core::{
    amethyst::{
        core::{components::Transform, SystemDesc, Time},
        derive::SystemDesc,
        ecs::{Entities, Join, Read, ReadStorage, System, SystemData, World, Write, WriteStorage},
        renderer::{ActiveCamera, Camera},
        shrev::{EventChannel, ReaderId},
        ui::{UiFinder, UiText},
    },
    clock::WorldTime,
    components::{IdleComponent, PawnComponent},
};

pub mod reactions;
pub use reactions::ExecuteRactionSystem;

pub mod movement;
pub use movement::{PathingMovementSystem, PathingMovementSystemDesc};

pub mod pawn_pickup;
pub use pawn_pickup::PawnPickupItemSystem;

pub mod input;

pub use ai::pathing::{PathingWorkSystem, PathingWorkSystemDesc};

pub mod tiles;
pub use tiles::TileEntitySystem;

pub mod world_view;

pub mod behavior;

#[derive(Default, SystemDesc)]
pub struct ShowCameraPosSystem;
impl<'s> System<'s> for ShowCameraPosSystem {
    type SystemData = (
        Read<'s, ActiveCamera>,
        Entities<'s>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, Transform>,
        WriteStorage<'s, UiText>,
        UiFinder<'s>,
    );

    fn run(
        &mut self,
        (active, entities, cameras, transforms, mut ui_text, finder): Self::SystemData,
    ) {
        let mut camera_join = (&cameras, &transforms).join();
        if let Some((_, camera_transform)) = active
            .entity
            .and_then(|a| camera_join.get(a, &entities))
            .or_else(|| camera_join.next())
        {
            if let Some(fps_entity) = finder.find("fps_text") {
                if let Some(fps_display) = ui_text.get_mut(fps_entity) {
                    let translation = camera_transform.global_matrix().column(3).xyz();
                    fps_display.text = format!(
                        "c = {}, {}, {}",
                        translation.x, translation.y, translation.z
                    );
                }
            }
        }
    }
}

#[derive(Default, SystemDesc)]
pub struct ShowIdlePawnsSystem;
impl<'s> System<'s> for ShowIdlePawnsSystem {
    type SystemData = (
        WriteStorage<'s, UiText>,
        ReadStorage<'s, PawnComponent>,
        ReadStorage<'s, IdleComponent>,
        UiFinder<'s>,
    );

    fn run(&mut self, (mut ui_text, pawn_storage, idle_storage, finder): Self::SystemData) {
        if let Some(entity) = finder.find("idle_text") {
            if let Some(display) = ui_text.get_mut(entity) {
                display.text = format!(
                    "{} Idle Pawns",
                    (&pawn_storage, &idle_storage).join().count()
                );
            }
        }
    }
}

use amethyst::input::InputEvent;
use core::input::{ActionBinding, BindingTypes};

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ChangeWorldSpeedEvent(pub f32);

pub struct ManageWorldSpeedSystem {
    reader: ReaderId<ChangeWorldSpeedEvent>,
    input_reader: ReaderId<InputEvent<BindingTypes>>,
    paused: Option<f32>,
}
impl<'s> System<'s> for ManageWorldSpeedSystem {
    type SystemData = (
        Read<'s, WorldTime>,
        Write<'s, Time>,
        Read<'s, EventChannel<ChangeWorldSpeedEvent>>,
        Read<'s, EventChannel<InputEvent<BindingTypes>>>,
    );

    fn run(&mut self, (world_time, mut amethyst_time, events, input_events): Self::SystemData) {
        for event in events.read(&mut self.reader) {
            amethyst_time.set_time_scale(event.0)
        }

        for event in input_events.read(&mut self.input_reader) {
            match event {
                InputEvent::ActionPressed(ActionBinding::Pause) => {
                    if self.paused.is_some() {
                        amethyst_time.set_time_scale(self.paused.unwrap());
                        self.paused = None;
                    } else {
                        self.paused = Some(amethyst_time.time_scale());
                        amethyst_time.set_time_scale(0.0);
                    }
                }
                _ => continue,
            }
        }

        if self.paused.is_some() && amethyst_time.time_scale() > 0.0 {
            self.paused = None;
        }

        world_time.elapse(&amethyst_time);
    }
}

#[derive(Default)]
pub struct ManageWorldSpeedSystemDesc;
impl<'a, 'b> SystemDesc<'a, 'b, ManageWorldSpeedSystem> for ManageWorldSpeedSystemDesc {
    fn build(self, world: &mut World) -> ManageWorldSpeedSystem {
        <ManageWorldSpeedSystem as System<'_>>::SystemData::setup(world);

        let reader = Write::<EventChannel<ChangeWorldSpeedEvent>>::fetch(world).register_reader();
        let input_reader =
            Write::<EventChannel<InputEvent<BindingTypes>>>::fetch(world).register_reader();

        // Set the game time speed to default
        Write::<Time>::fetch(world).set_time_scale(core::clock::scales::normal());

        ManageWorldSpeedSystem {
            reader,
            input_reader,
            paused: None,
        }
    }
}
