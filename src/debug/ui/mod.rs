use amethyst::core::ecs::{SystemData, World, Write};

pub mod buildings;
pub mod objects;
pub mod pawn;
pub mod work;

use amethyst_imgui::imgui::{ImString, Ui};

pub fn setup(world: &mut World) {
    pawn::setup(world);
    objects::setup(world);
    buildings::setup(world);
    work::setup(world);
    GlobalState::setup(world);
}

pub fn draw(ui: &Ui, world: &mut World) {
    crate::debug::ui::pawn::draw(&ui, world);
    crate::debug::ui::objects::draw(&ui, world);
    crate::debug::ui::buildings::draw(&ui, world);
    crate::debug::ui::work::draw(&ui, world);
}

pub struct DebugState {
    pub action_name: ImString,
}
impl Default for DebugState {
    fn default() -> Self {
        Self {
            action_name: ImString::with_capacity(128),
        }
    }
}
type GlobalState<'a> = (Write<'a, DebugState>);
