use amethyst::{
    core::{
        components::Transform,
        ecs::{Entities, Join, ReadStorage, SystemData, World, WriteStorage},
    },
    tiles::{Map, TileMap},
};
use core::components::{PropertiesComponent, TilePosition};

use amethyst_imgui::imgui::{self, im_str, Condition, ImString, Ui};

type PropertiesData<'a> = (
    Entities<'a>,
    WriteStorage<'a, PropertiesComponent>,
    ReadStorage<'a, Transform>,
    ReadStorage<'a, TilePosition>,
    ReadStorage<'a, TileMap<core::tiles::region::RegionTile>>,
);

pub fn setup(world: &mut World) {
    PropertiesData::setup(world);
}

pub fn draw(ui: &Ui, world: &mut World) {
    imgui::Window::new(im_str!("Object"))
        .size([500.0, 1000.0], Condition::FirstUseEver)
        .build(ui, || {
            let (entities, property_storage, transform_storage, tilepos_storage, tilemap_storage) =
                PropertiesData::fetch(&world);

            if let Some(tile_map) = (&tilemap_storage).join().next() {
                for (entity, properties, transform) in
                    (&entities, &property_storage, &transform_storage).join()
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

                        ui.columns(
                            4,
                            &ImString::from(format!("Properties ##{}", entity.id())),
                            true,
                        );
                        for property in properties.iter() {
                            ui.text(format!("{:?}", property).as_str());
                            ui.next_column();
                        }
                        ui.columns(1, im_str!(""), false);
                    }
                }
            }
        });
}
