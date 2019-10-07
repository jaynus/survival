#![deny(clippy::pedantic, clippy::all, unused_imports)]
#![allow(dead_code, unused_variables)]

use crate::initializers;
use amethyst::{
    assets::ProgressCounter,
    core::{
        ecs::{Builder, ReadStorage, SystemData, World, WorldExt},
        math::Vector3,
        Transform,
    },
    input::{is_close_requested, is_key_down},
    tiles::TileMap,
    ui::UiCreator,
    winit, GameData, {StateData, StateEvent, Trans},
};
use amethyst_imgui::imgui::{self, im_str, ImString};
use core::{image, rand::SeedableRng, settings::GraphicsSettings, tiles::world::WorldTile};
use map::{
    world::{StandardGenerator, StandardSettings},
    Generator,
};

#[derive(Default)]
pub struct WorldGeneration {
    progress: ProgressCounter,
    seed: ImString,
    generator: usize,
    ui_manager: Option<crate::ui::UiManager>,
}
impl WorldGeneration {
    fn do_generate(&mut self, world: &mut World) -> Result<(), failure::Error> {
        let sprite_sheet = {
            world
                .read_resource::<GraphicsSettings>()
                .sprite_sheets
                .get("default_map")
                .map(|v| (*v).clone())
        };

        let dims = Vector3::<u32>::new(1024, 1024, 1);
        log::info!("Allocating map of size: {}", dims);

        let mut world_map = TileMap::<WorldTile>::new(dims, Vector3::new(16, 16, 1), sprite_sheet);

        // TODO: for testing
        let app_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = app_root.join("worldgen/output/dt_spade_interp.png");
        println!("{:?}", path);

        if let image::DynamicImage::ImageLuma8(image) = image::open(path).unwrap() {
            let settings = StandardSettings::new(&image);
            let mut generator = StandardGenerator::new(settings);

            let seed = map::utils::seed_from_str(self.seed.to_str());
            let mut rng =
                core::rand_xorshift::XorShiftRng::from_seed(*arrayref::array_ref![&seed, 0, 16]);

            generator.execute(&mut world_map, world, &mut rng).unwrap();
        }
        println!("Generated, adding");
        world
            .create_entity()
            .with(world_map)
            .with(Transform::default())
            .build();

        Ok(())
    }
}

type SetupData<'a> = (ReadStorage<'a, TileMap<WorldTile>>,);

impl<'a, 'b> amethyst::State<GameData<'a, 'b>, StateEvent> for WorldGeneration {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        SetupData::setup(world);

        world.exec(|mut creator: UiCreator<'_>| {
            creator.create("ui/worldgen.ron", &mut self.progress);
        });

        self.seed = "balls".to_string().into();

        let _camera = initializers::camera(world, &mut self.progress);

        self.ui_manager = Some(
            crate::ui::UiManager::default()
                .add(crate::ui::embark::EmbarkWindow::default(), false)
                .build(world),
        );
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

            imgui::Window::new(im_str!("World Generation"))
                //.size([500.0, 1000.0], imgui::Condition::FirstUseEver)
                .build(ui, || {
                    if ui.button(im_str!("Regenerate Random"), [0.0, 0.0]) {
                        self.do_generate(world).unwrap();
                    }
                    imgui::ComboBox::new(im_str!("combo")).build_simple_string(
                        ui,
                        &mut self.generator,
                        &[im_str!("Standard")],
                    );
                    ui.input_text(im_str!("Seed"), &mut self.seed).build();
                    ui.separator();
                    if ui.button(im_str!("Reload Definitions"), [0.0, 0.0]) {
                        if let Err(e) = crate::loaders::reload_defs(world) {
                            log::error!("Reloading of definitions FAILED!: {:?}", e);
                        }
                    }
                });
        });

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
