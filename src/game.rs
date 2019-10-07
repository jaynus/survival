#![deny(clippy::pedantic, clippy::all)]
#![allow(dead_code, unused_variables)]
#![feature(trait_alias)]

pub use survival::*;

use amethyst::{
    assets::HotReloadBundle,
    core::TransformBundle,
    input::InputBundle,
    renderer::{
        plugins::RenderDebugLines, types::DefaultBackend, Format, RenderToWindow, RenderingBundle,
    },
    tiles::{MortonEncoder2D, RenderTiles2D},
    ui::{RenderUi, UiBundle},
    utils::application_root_dir,
    window::ScreenDimensions,
    Application, GameDataBuilder,
};
use amethyst_imgui::RenderImgui;
use core::tiles::DrawRegionTileBounds;

fn prepare_logging() {
    amethyst::Logger::from_config(amethyst::LoggerConfig::default())
        .level_for("gfx_backend_vulkan", log::LevelFilter::Error)
        .level_for("rendy_graph", log::LevelFilter::Warn)
        .level_for("core", log::LevelFilter::Warn)
        .level_for("core::fsm", log::LevelFilter::Warn)
        .level_for("map", log::LevelFilter::Warn)
        .level_for("amethyst_tiles", log::LevelFilter::Warn)
        .level_for("body", log::LevelFilter::Warn)
        .level_for("ai", log::LevelFilter::Trace)
        //.level_for("ai::pathing", log::LevelFilter::Warn)
        //.level_for("ai::tasks", log::LevelFilter::Warn)
        //.level_for("ai::behavior", log::LevelFilter::Warn)
        .level_for("ai::behavior::nodes::reaction", log::LevelFilter::Trace)
        .level_for("survival", log::LevelFilter::Warn)
        .level_for("survival::systems", log::LevelFilter::Warn)
        .level_for("survival::systems::movement", log::LevelFilter::Warn)
        .level_for(
            "survival::systems::input::selection",
            log::LevelFilter::Trace,
        )
        .level_for("survival::systems::reactions", log::LevelFilter::Trace)
        .level_for("survival_game", log::LevelFilter::Trace)
        .level_for("survival::states", log::LevelFilter::Trace)
        .start();
}

fn main() -> amethyst::Result<()> {
    prepare_logging();

    let app_root = application_root_dir()?;
    let assets_directory = app_root.join("resources/assets");
    let display_config_path = app_root.join("resources/display_config.ron");

    let app_builder = Application::build(assets_directory, states::Loading::default())?;

    let game_data = GameDataBuilder::default()
        .with_barrier()
        .with_bundle(TransformBundle::new())?
        .with_bundle(HotReloadBundle::default())?
        .with_bundle(
            InputBundle::<core::input::BindingTypes>::new()
                .with_bindings_from_file("resources/input.ron")?,
        )?
        .with_bundle(systems::input::InputBundle::default())?
        .with_system_desc(
            systems::ShowCameraPosSystem::default(),
            "ShowCameraPosSystem",
            &[],
        )
        .with_system_desc(
            systems::tiles::TileEntitySystem::default(),
            "TileEntitySystem",
            &[],
        )
        .with_system_desc(
            systems::PathingMovementSystemDesc::default(),
            "PathingMovementSystem",
            &[],
        )
        .with_system_desc(
            systems::ShowIdlePawnsSystem::default(),
            "ShowIdlePawnsSystem",
            &[],
        )
        .with_system_desc(
            crate::systems::ExecuteRactionSystem::default(),
            "ExecuteRactionSystem",
            &[],
        )
        .with_system_desc(
            systems::PawnPickupItemSystem::default(),
            "PawnPickupItemSystem",
            &[],
        )
        .with_system_desc(
            systems::PathingWorkSystemDesc::default(),
            "PathingWorkSystem",
            &[],
        )
        .with_system_desc(
            systems::ManageWorldSpeedSystemDesc::default(),
            "ManageWorldSpeedSystem",
            &[],
        )
        .with_bundle(systems::world_view::WorldViewBundle::default())?
        .with_bundle(psyche::systems::PsycheBundle::default())?
        .with_bundle(body::bundle::BodyBundle::default())?
        .with_bundle(UiBundle::<core::input::BindingTypes>::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                .with_plugin(RenderUi::default())
                .with_plugin(RenderImgui::<core::input::BindingTypes>::default())
                .with_plugin(renderer::RenderSprites::default())
                .with_plugin(RenderTiles2D::<
                    core::tiles::region::RegionTile,
                    MortonEncoder2D,
                    DrawRegionTileBounds,
                >::default())
                .with_plugin(RenderTiles2D::<
                    core::tiles::world::WorldTile,
                    MortonEncoder2D,
                    DrawRegionTileBounds,
                >::default())
                .with_plugin(RenderDebugLines::default()),
        )?;
    //.with_bundle(ai::tasks::TaskBundle::default())?
    //.with_bundle(ai::behavior::BehaviorTreeBundle::default())?;

    let mut game = app_builder.build(game_data)?;
    game.run();
    Ok(())
}

#[derive(Default)]
struct ExampleGraph {
    dimensions: Option<ScreenDimensions>,
    surface_format: Option<Format>,
    dirty: bool,
}
