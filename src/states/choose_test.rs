#![deny(clippy::pedantic, clippy::all, unused_imports)]
#![allow(dead_code, unused_variables)]

use amethyst::{
    assets::ProgressCounter,
    input::{is_close_requested, is_key_down},
    winit, GameData, {StateData, StateEvent, Trans},
};

use amethyst_imgui::imgui::{self, im_str, ImString};
use num_traits::FromPrimitive;
use strum::IntoEnumIterator;

#[derive(
    strum_macros::AsRefStr,
    strum_macros::EnumIter,
    strum_macros::ToString,
    num_derive::FromPrimitive,
)]
#[repr(i32)]
pub enum CurrentStateSelection {
    None = 0,
    TestRegion = 1,
    WorldGen = 2,
}

#[derive(Default)]
pub struct ChooseTest {
    pub selected_state: usize,
    progress: ProgressCounter,
}
impl<'a, 'b> amethyst::State<GameData<'a, 'b>, StateEvent> for ChooseTest {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        world.insert(CurrentStateSelection::None);
    }
    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit).
    fn update(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
    ) -> Trans<GameData<'a, 'b>, StateEvent> {
        let StateData { world, .. } = data;
        data.data.update(world);

        amethyst_imgui::with(|ui| {
            //ui.show_demo_window(&mut t);

            imgui::Window::new(im_str!("Debug Choose Test State")).build(ui, || {
                imgui::ComboBox::new(im_str!("State")).build_simple_string(
                    ui,
                    &mut self.selected_state,
                    &CurrentStateSelection::iter()
                        .map(|v| ImString::from(v.to_string()))
                        .collect::<Vec<_>>()
                        .iter()
                        .collect::<Vec<_>>(),
                );
            });

            *world.fetch_mut::<CurrentStateSelection>() =
                CurrentStateSelection::from_usize(self.selected_state).unwrap();
        });

        match *world.fetch::<CurrentStateSelection>() {
            CurrentStateSelection::None => Trans::None,
            CurrentStateSelection::TestRegion => {
                Trans::Push(Box::new(crate::states::TestRegion::default()))
            }
            CurrentStateSelection::WorldGen => {
                Trans::Push(Box::new(crate::states::WorldGeneration::default()))
            }
        }
    }

    fn shadow_update(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;
        amethyst_imgui::with(|ui| {
            crate::debug::ui::draw(ui, world);
        });
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
