use amethyst::{
    core::{
        components::Transform,
        ecs::{Entities, Join, ReadStorage, SystemData, World, Write, WriteStorage},
    },
    tiles::{Map, TileMap},
};
use amethyst_imgui::imgui::{self, im_str, Condition, ImString, Ui};
use core::components::{BuildingComponent, PropertiesComponent, TilePosition};

type PropertiesData<'a> = (
    Entities<'a>,
    WriteStorage<'a, PropertiesComponent>,
    ReadStorage<'a, BuildingComponent>,
    ReadStorage<'a, Transform>,
    ReadStorage<'a, TilePosition>,
    ReadStorage<'a, TileMap<core::tiles::region::RegionTile>>,
    Write<'a, DebugBuildingWindowState>,
);

pub fn setup(world: &mut World) {
    PropertiesData::setup(world);
}

#[derive(Default)]
pub struct DebugBuildingWindowState {
    _selected_building: usize,
}

pub fn draw(ui: &Ui, world: &mut World) {
    imgui::Window::new(im_str!("Buildings"))
        .size([500.0, 1000.0], Condition::FirstUseEver)
        .build(ui, || {
            let (
                entities,
                property_storage,
                building_storage,
                transform_storage,
                tilepos_storage,
                tilemap_storage,
                _,
            ) = PropertiesData::fetch(&world);

            if let Some(tile_map) = (&tilemap_storage).join().next() {
                for (entity, _, transform, _) in (
                    &entities,
                    &property_storage,
                    &transform_storage,
                    &building_storage,
                )
                    .join()
                {
                    let obj_id = format!("ID: {}", entity.id());

                    if ui
                        .collapsing_header(&ImString::from(obj_id))
                        .default_open(true)
                        .build()
                    {
                        let coord = tile_map.to_tile(transform.translation()).unwrap();
                        ui.text(&format!("loc: ({}, {}, {})", coord.x, coord.y, coord.z));

                        if let Some(tile_position) = tilepos_storage.get(entity) {
                            ui.text(&format!(
                                "tp: ({}, {}, {})",
                                tile_position.0.x, tile_position.0.y, tile_position.0.z
                            ));
                        } else {
                            ui.text("NO TP");
                        }
                    }
                }
            }
        });
}
