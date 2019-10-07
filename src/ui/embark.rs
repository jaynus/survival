use crate::ui::ImguiDrawable;
use amethyst::{
    ecs::{ReadExpect, ReadStorage, SystemData, World},
    tiles::TileMap,
};
use amethyst_imgui::imgui;
use core::{embark::EmbarkSettings, tiles::world::WorldTile};

type EmbarkData<'a> = (
    ReadExpect<'a, EmbarkSettings>,
    ReadStorage<'a, TileMap<WorldTile>>,
);

#[derive(Debug, Default)]
pub struct EmbarkWindow {}
impl ImguiDrawable for EmbarkWindow {
    fn setup(&mut self, world: &mut World) {
        EmbarkData::setup(world);
    }

    fn name(&self) -> &'static str {
        "EmbarkWindow"
    }

    fn draw(&mut self, ui: &imgui::Ui, world: &mut World) {
        imgui::Window::new(imgui::im_str!("Embark##UI"))
            .position([0.0, 0.0], imgui::Condition::FirstUseEver)
            .size([500.0, 500.0], imgui::Condition::FirstUseEver)
            .build(ui, || {
                let (embark_settings, _) = EmbarkData::fetch(world);

                if let Some(region) = embark_settings.region {
                    ui.text(format!("Start: {}, {}", region.min.x, region.min.y,));
                    ui.text(format!("Start: {}, {}", region.max.x, region.max.y,));
                }
            });
    }
}
