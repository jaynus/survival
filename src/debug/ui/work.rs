use super::DebugState;
use amethyst::core::ecs::{Read, ReadExpect, SystemData, World, Write};
use amethyst_imgui::imgui::{self, im_str, Condition, ImString, Ui};
use core::{
    defs::{DefinitionStorage, Named},
    fsm::TaskCategory,
};

type WorkData<'a> = (Write<'a, DebugState>,);

pub fn setup(world: &mut World) {
    WorkData::setup(world);
}

pub fn draw(ui: &Ui, world: &mut World) {
    imgui::Window::new(im_str!("Work"))
        .size([500.0, 1000.0], Condition::FirstUseEver)
        .build(ui, || {});
}
