#![deny(clippy::pedantic, clippy::all, unused_imports)]
#![allow(dead_code, unused_variables)]

use crate::initializers;
use amethyst::{
    assets::{Handle, ProgressCounter},
    core::{
        ecs::{Builder, Join, ReadStorage, SystemData, World, WorldExt},
        math::{Point3, Vector3},
        Time, Transform,
    },
    input::{is_close_requested, is_key_down},
    renderer::{SpriteSheet, Transparent},
    tiles::{Map, TileMap},
    ui::UiCreator,
    winit, GameData, {StateData, StateEvent, Trans},
};
use amethyst_imgui::imgui::{self, im_str, ImString};
use core::SpriteRender;
use core::{
    components::{BuildingComponent, PropertiesComponent, TilePosition},
    defs::{building::BuildingDefinition, DefinitionStorage},
    num_traits::FromPrimitive,
    rand::SeedableRng,
    rand_xorshift::XorShiftRng,
    settings::GraphicsSettings,
    tiles::region::RegionTile,
};
use map::{
    region::random::{RandomGenerator, RandomSettings},
    Generator,
};

pub fn create_test_trees<R>(world: &mut World, rng: &mut R)
where
    R: core::rand::Rng,
{
    use core::{amethyst::tiles::MapStorage, defs::property::MovementFlags};

    for i in 0..5000 {
        let tile_position = {
            let tilemaps = &world.read_component::<TileMap<RegionTile>>();
            let map = (tilemaps).join().next().unwrap();

            let coord = Point3::new(
                rng.gen_range(0, map.dimensions().x - 1),
                rng.gen_range(0, map.dimensions().y - 1),
                0,
            );

            if let Some(tile) = map.get(&coord) {
                if !tile.passable(MovementFlags::Walk) {
                    continue;
                }
            }

            coord
        };

        core::initializers::spawn_foliage("oak tree", &tile_position, world);
    }
}

pub fn create_test_axe((x, y): (u32, u32), world: &mut World) {
    crate::initializers::spawn_item("Axe", Some(Point3::new(x, y, 0)), None, None, None, world);
}

pub fn create_test_timberyard(
    (x, y): (u32, u32),
    world: &mut World,
    sprite_sheet: &Option<Handle<SpriteSheet>>,
) {
    let location = {
        let tilemaps = &world.read_component::<TileMap<RegionTile>>();
        let map = (tilemaps).join().next().unwrap();

        let mut location = Transform::default();
        location.set_translation(map.to_world(&Point3::new(x, y, 0)));
        location
    };

    log::trace!(
        "Placing new test building ('timberyard') @ : {:?}",
        location
    );

    let building = {
        let buildings = world.fetch::<DefinitionStorage<BuildingDefinition>>();
        BuildingComponent::new(buildings.get_id("timberyard").unwrap(), &buildings)
    };

    let mut builder = world
        .create_entity()
        .with(PropertiesComponent::default())
        .with(Transparent)
        .with(building)
        .with(TilePosition::default())
        .with(location);

    if let Some(sheet_handle) = sprite_sheet {
        builder = builder.with(SpriteRender {
            sprite_sheet: sheet_handle.clone(),
            sprite_number: 71,
            z_modifier: core::z_level_modifiers::BUILDING,
        });
    }

    builder.build();
}

type SetupData<'a> = (ReadStorage<'a, BuildingComponent>,);

#[derive(Default)]
pub struct TestRegion {
    progress: ProgressCounter,
    seed: ImString,
    game_speed_selection: i32,
    last_game_speed_selection: i32,
    render_mode: usize,
    last_render_mode: usize,
    generator: usize,
    ui_manager: Option<crate::ui::UiManager>,
}
impl TestRegion {
    fn do_generate(&mut self, world: &mut World) -> Result<(), failure::Error> {
        let sprite_sheet = {
            world
                .read_resource::<GraphicsSettings>()
                .sprite_sheets
                .get("default_map")
                .map(|v| (*v).clone())
        };

        let dims = Vector3::<u32>::new(64, 64, 64);
        log::info!("Allocating map of size: {}", dims);
        log::info!("Tile size = {}b", std::mem::size_of::<RegionTile>());
        log::info!(
            "Allocation: {}b",
            dims.x as usize * dims.y as usize * dims.z as usize * std::mem::size_of::<RegionTile>()
        );

        let mut map =
            TileMap::<RegionTile>::new(dims, Vector3::new(16, 16, 1), sprite_sheet.clone());

        let mut gen = RandomGenerator::new(RandomSettings::default());
        let seed = map::utils::seed_from_str(self.seed.to_str());
        let mut rng = XorShiftRng::from_seed(*arrayref::array_ref![&seed, 0, 16]);
        gen.execute(&mut map, world, &mut rng)?;

        world
            .create_entity()
            .with(map)
            .with(Transform::default())
            .build();

        let (pawn_1_pos, pawn_2_pos, timberyard_pos) = {
            let tilemaps = &world.read_component::<TileMap<RegionTile>>();
            let map = (tilemaps).join().next().unwrap();

            (
                Point3::new(map.dimensions().x / 2, map.dimensions().y / 2, 0),
                Point3::new(map.dimensions().x / 2 - 10, map.dimensions().y / 2 - 10, 0),
                (map.dimensions().x / 2 + 5, map.dimensions().y / 2 + 5),
            )
        };

        crate::initializers::spawn_pawn("human", &pawn_1_pos, world);
        crate::initializers::spawn_pawn("human", &pawn_2_pos, world);

        create_test_axe((0, 0), world);
        create_test_axe((0, 4), world);

        create_test_timberyard(timberyard_pos, world, &sprite_sheet);

        create_test_trees(world, &mut rng);

        Ok(())
    }
}

impl<'a, 'b> amethyst::State<GameData<'a, 'b>, StateEvent> for TestRegion {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        world.exec(|mut creator: UiCreator<'_>| {
            creator.create("ui/test_region.ron", &mut self.progress);
        });

        self.seed = "balls".to_string().into();

        let _camera = initializers::camera(world, &mut self.progress);

        self.ui_manager = Some(
            crate::ui::UiManager::default()
                .add(crate::ui::SelectionWindow::default(), false)
                .add(crate::ui::BuildMenuWindow::default(), false)
                .add(crate::ui::PawnWindow::default(), false)
                .build(world),
        );

        SetupData::setup(world);
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit).
    #[allow(clippy::cast_possible_truncation)]
    fn update(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
    ) -> Trans<GameData<'a, 'b>, StateEvent> {
        let StateData { world, .. } = data;
        data.data.update(world);

        amethyst_imgui::with(|ui| {
            self.ui_manager.as_mut().unwrap().draw(ui, world);

            imgui::Window::new(im_str!("Test Region"))
                //.size([500.0, 1000.0], imgui::Condition::FirstUseEver)
                .build(ui, || {
                    if ui.button(im_str!("Regenerate Random"), [0.0, 0.0]) {
                        self.do_generate(world).unwrap();
                    }
                    imgui::ComboBox::new(im_str!("combo")).build_simple_string(
                        ui,
                        &mut self.generator,
                        &[im_str!("Random")],
                    );
                    ui.input_text(im_str!("Seed"), &mut self.seed).build();
                    ui.separator();
                    if ui.button(im_str!("Reload Definitions"), [0.0, 0.0]) {
                        if let Err(e) = crate::loaders::reload_defs(world) {
                            log::error!("Reloading of definitions FAILED!: {:?}", e);
                        }
                    }
                    let mut time_scale = world.fetch::<Time>().time_scale();
                    imgui::Slider::new(im_str!("Time Scale"), 0.0..=1000.0)
                        .build(ui, &mut time_scale);
                    world.fetch_mut::<Time>().set_time_scale(time_scale);

                    ui.text(im_str!("Game Speed"));
                    ui.radio_button(im_str!("Normal"), &mut self.game_speed_selection, 0);
                    ui.same_line(0.0);
                    ui.radio_button(im_str!("Fast"), &mut self.game_speed_selection, 1);
                    ui.same_line(0.0);
                    ui.radio_button(im_str!("Fastest"), &mut self.game_speed_selection, 2);
                    if self.game_speed_selection != self.last_game_speed_selection {
                        use crate::systems::ChangeWorldSpeedEvent;
                        use amethyst::shrev::EventChannel;
                        use core::clock::scales;

                        let speed_event = match self.game_speed_selection {
                            0 => ChangeWorldSpeedEvent(scales::normal()),
                            1 => ChangeWorldSpeedEvent(scales::fast()),
                            2 => ChangeWorldSpeedEvent(scales::fastest()),
                            _ => unimplemented!(),
                        };
                        world
                            .fetch_mut::<EventChannel<ChangeWorldSpeedEvent>>()
                            .single_write(speed_event);

                        self.last_game_speed_selection = self.game_speed_selection;
                    }

                    if ui.button(im_str!("Reset Camera Z"), [0.0, 0.0]) {
                        use amethyst::{core::ecs::WriteStorage, renderer::Camera};
                        let (mut transform_storage, camera_storage) =
                            <(WriteStorage<'_, Transform>, ReadStorage<'_, Camera>)>::fetch(world);
                        for (transform, _) in (&mut transform_storage, &camera_storage).join() {
                            transform.set_translation_z(0.9);
                        }
                    }

                    imgui::ComboBox::new(im_str!("Render Mode")).build_simple_string(
                        ui,
                        &mut self.render_mode,
                        &[im_str!("Normal"), im_str!("Pathing")],
                    );
                    if self.render_mode != self.last_render_mode {
                        world
                            .fetch_mut::<core::settings::GraphicsSettings>()
                            .map_render_mode =
                            core::settings::RegionMapRenderMode::from_u8(self.render_mode as u8)
                                .unwrap();

                        self.last_render_mode = self.render_mode;
                    }
                });
        });

        // get the current mouse pos, project it to world, then project it to tile, then display it
        //mouse_pos

        Trans::None
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
