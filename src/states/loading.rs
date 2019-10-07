#![deny(clippy::pedantic, clippy::all, unused_imports)]
#![allow(dead_code, unused_variables)]

use crate::initializers;
use crate::loaders;
use amethyst::{
    assets::ProgressCounter,
    core::{ecs::SystemData, math::Vector3},
    input::{is_close_requested, is_key_down},
    winit, GameData, {StateData, StateEvent, Trans},
};
use std::collections::HashMap;

#[derive(Default)]
pub struct Loading {
    progress: ProgressCounter,
}

impl<'a, 'b> amethyst::State<GameData<'a, 'b>, StateEvent> for Loading {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        log::trace!("enter?");
        let world = data.world;
        let mut sprite_sheets = HashMap::new();

        world.insert(core::embark::EmbarkSettings::default());

        core::components::AllComponents::setup(world);

        crate::debug::ui::setup(world);

        sprite_sheets.insert(
            "default_map".to_string(),
            initializers::load_sprite_sheet(
                world,
                "spritesheets/Kelora_16x16.png",
                "spritesheets/Kelora_16x16.ron",
                &mut self.progress,
            ),
        );

        sprite_sheets.insert(
            "buildings".to_string(),
            initializers::load_sprite_sheet(
                world,
                "buildings.png",
                "buildings.ron",
                &mut self.progress,
            ),
        );

        world.insert(core::settings::GraphicsSettings {
            tile_dimensions: Vector3::new(16, 16, 1),
            tilesets: Vec::new(),
            sprite_sheets,
            map_render_mode: core::settings::RegionMapRenderMode::default(),
        });

        if let Err(e) = loaders::assets(world) {
            log::error!("Failed to laod assets! Try to reload: {:?}", e);
        }
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit).
    fn update(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
    ) -> Trans<GameData<'a, 'b>, StateEvent> {
        let StateData { world, .. } = data;
        data.data.update(world);

        if self.progress.is_complete() {
            log::trace!("Loading complete, transitioning to ChooseTest");
            Trans::Switch(Box::new(crate::states::ChooseTest::default()))
        } else {
            Trans::None
        }
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> Trans<GameData<'a, 'b>, StateEvent> {
        let StateData { world, .. } = data;
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, winit::VirtualKeyCode::Escape) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }
}
