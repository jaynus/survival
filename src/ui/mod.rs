#![allow(dead_code)]
use amethyst::{
    core::components::Transform,
    ecs::{Entities, Entity, Join, LazyUpdate, Read, ReadExpect, ReadStorage, SystemData, World},
    shrev::EventChannel,
    tiles::{Map, TileMap},
};
use amethyst_imgui::imgui::{self, im_str, ImString};
use std::collections::HashMap;

use crate::components;
use core::{
    defs::{
        building::BuildingDefinition,
        reaction::{ReactionDefinition, Reagent},
        DefinitionComponent, DefinitionStorage, Named,
    },
    input::{InputState, PlayerInputEvent, SelectionData},
    shrinkwraprs::Shrinkwrap,
    tiles::{region::RegionTile, TileEntityStorage},
    ItemHierarchy,
};

pub mod embark;

pub trait ImguiDrawable: std::fmt::Debug + Send + Sync {
    fn name(&self) -> &'static str;

    fn setup(&mut self, _world: &mut World) {}

    fn draw(&mut self, ui: &imgui::Ui, world: &mut World);

    fn on_toggle_hidden(&mut self, hidden: bool) -> bool {
        hidden
    }
}

#[derive(Debug, Default)]
pub struct SelectionWindow {}
impl SelectionWindow {
    fn draw_tile(&mut self, _: &RegionTile, _: &imgui::Ui, _: &World) {}

    fn draw_item(&mut self, entity: Entity, ui: &imgui::Ui, world: &World) {
        let (item_defs, item_comp_storage) = <(
            ReadExpect<'_, DefinitionStorage<core::defs::item::ItemDefinition>>,
            ReadStorage<'_, components::ItemComponent>,
        )>::fetch(world);
        let def = item_comp_storage
            .get(entity)
            .unwrap()
            .fetch_def(&item_defs)
            .unwrap();
        ui.group(|| {
            ui.text(&format!("ITEM: {}", def.name()));
        });
    }

    fn draw_foliage(&mut self, entity: Entity, ui: &imgui::Ui, world: &World) {
        let (foliage_defs, foliage_comp_storage) = <(
            ReadExpect<'_, DefinitionStorage<core::defs::foliage::FoliageDefinition>>,
            ReadStorage<'_, components::FoliageComponent>,
        )>::fetch(world);
        let def = foliage_comp_storage
            .get(entity)
            .unwrap()
            .fetch_def(&foliage_defs)
            .unwrap();

        ui.group(|| {
            ui.text(&format!("FOLIAGE: {}", def.name()));
        });
    }

    fn draw_building(&mut self, entity: Entity, ui: &imgui::Ui, world: &World) {
        let (building_defs, building_comp_storage) = <(
            ReadExpect<'_, DefinitionStorage<core::defs::building::BuildingDefinition>>,
            ReadStorage<'_, components::BuildingComponent>,
        )>::fetch(world);
        let def = building_comp_storage
            .get(entity)
            .unwrap()
            .fetch_def(&building_defs)
            .unwrap();

        ui.group(|| {
            ui.text(&format!("BUILDING: {}", def.name()));
        });
    }

    fn draw_pawn(&mut self, _: Entity, ui: &imgui::Ui, _: &World) {
        ui.group(|| {
            ui.text("pawn");
        });
    }
}
impl ImguiDrawable for SelectionWindow {
    fn name(&self) -> &'static str {
        "SelectionWindow"
    }
    fn setup(&mut self, world: &mut World) {
        <(ReadStorage<'_, components::TypeTagComponent>)>::setup(world);
    }

    #[allow(unreachable_patterns)]
    fn draw(&mut self, ui: &imgui::Ui, world: &mut World) {
        use core::components::TypeTagComponent;

        imgui::Window::new(imgui::im_str!("##Selection"))
            .position([0.0, 0.0], imgui::Condition::Always)
            .size([200.0, 200.0], imgui::Condition::FirstUseEver)
            .build(ui, || {
                let input_state = world.fetch::<InputState>();

                if ui
                    .collapsing_header(im_str!("Under Mouse"))
                    .default_open(true)
                    .build()
                {
                    let (entities, tile_entities, map_storage, type_tag_storage) =
                        <(
                            Entities<'_>,
                            Read<TileEntityStorage>,
                            ReadStorage<'_, TileMap<core::tiles::region::RegionTile>>,
                            ReadStorage<'_, components::TypeTagComponent>,
                        )>::fetch(world);

                    if let Some(map) = (&map_storage).join().next() {
                        if let Some(tile_pos) =
                            map.to_tile(&input_state.mouse_world_position.coords)
                        {
                            ui.text(&format!("Tile: ({}. {})", tile_pos.x, tile_pos.y));
                            if let Some(tile_entities) = tile_entities.get_point(&tile_pos, &map) {
                                for (entity, tag, _) in
                                    (&entities, &type_tag_storage, tile_entities).join()
                                {
                                    match tag {
                                        TypeTagComponent::Building => {
                                            self.draw_building(entity, ui, world)
                                        }
                                        TypeTagComponent::Foliage => {
                                            self.draw_foliage(entity, ui, world)
                                        }
                                        TypeTagComponent::Pawn(_) => {
                                            self.draw_pawn(entity, ui, world)
                                        }
                                        TypeTagComponent::Item => self.draw_item(entity, ui, world),
                                        _ => {
                                            ui.text("Unsupported entity");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if ui
                    .collapsing_header(im_str!("Selected Pawns"))
                    .default_open(true)
                    .build()
                {
                    if let Some(selection_data) = input_state.selection.as_ref() {
                        if let SelectionData::PawnGroup { entities, .. } = selection_data {
                            let (_, pawn_storage, _, _) =
                                <(
                                    ReadExpect<'_, ItemHierarchy>,
                                    ReadStorage<'_, components::PawnComponent>,
                                    ReadStorage<'_, components::ItemComponent>,
                                    ReadStorage<'_, components::ItemParentComponent>,
                                )>::fetch(world);
                            for (_, pawn) in (entities, &pawn_storage).join() {
                                ui.text(&format!("{:?}", pawn.name));
                                ui.same_line(0.0);
                            }
                        }
                    }
                }
            });
    }
}

type PawnData<'a> = (
    Read<'a, LazyUpdate>,
    Entities<'a>,
    ReadStorage<'a, components::PawnComponent>,
    ReadStorage<'a, components::IdleComponent>,
    ReadStorage<'a, components::ItemComponent>,
    ReadStorage<'a, Transform>,
    ReadStorage<'a, TileMap<core::tiles::region::RegionTile>>,
    ReadStorage<'a, components::BodyComponent>,
    ReadStorage<'a, components::RaceComponent>,
    ReadStorage<'a, components::PyscheNeedsComponent>,
    ReadStorage<'a, components::AttributesComponent>,
);

#[derive(Debug, Default)]
pub struct PawnWindow {}
impl ImguiDrawable for PawnWindow {
    fn setup(&mut self, world: &mut World) {
        PawnData::setup(world);
    }

    fn name(&self) -> &'static str {
        "PawnWindow"
    }

    fn draw(&mut self, ui: &imgui::Ui, _world: &mut World) {
        imgui::Window::new(imgui::im_str!("Pawns##UI"))
            .position([0.0, 0.0], imgui::Condition::FirstUseEver)
            .size([500.0, 500.0], imgui::Condition::FirstUseEver)
            .build(ui, || {});
    }
}

#[derive(Debug, Default)]
pub struct BuildMenuWindow {
    cur_selected_building: usize,
    selected_recipe: usize,
}
impl ImguiDrawable for BuildMenuWindow {
    fn name(&self) -> &'static str {
        "BuildWindow"
    }

    fn setup(&mut self, world: &mut World) {
        <(
            ReadExpect<'_, DefinitionStorage<core::defs::building::BuildingDefinition>>,
            Read<'_, EventChannel<PlayerInputEvent>>,
        )>::setup(world);
    }

    fn draw(&mut self, ui: &imgui::Ui, world: &mut World) {
        imgui::Window::new(imgui::im_str!("Build##UI"))
            .position([0.0, 0.0], imgui::Condition::FirstUseEver)
            .size([500.0, 500.0], imgui::Condition::FirstUseEver)
            .build(ui, || {
                if ui
                    .collapsing_header(im_str!("Tools"))
                    .default_open(true)
                    .build()
                {
                    imgui::ComboBox::new(im_str!("Building")).build_simple_string(
                        ui,
                        &mut self.cur_selected_building,
                        &world
                            .fetch::<DefinitionStorage<core::defs::building::BuildingDefinition>>()
                            .iter()
                            .map(|d| ImString::from(d.name().to_string()))
                            .collect::<Vec<_>>()
                            .iter()
                            .collect::<Vec<_>>(),
                    );
                    if ui.button(im_str!("Select"), [0.0, 0.0]) {
                        let buildings = world
                            .fetch::<DefinitionStorage<core::defs::building::BuildingDefinition>>()
                            .iter()
                            .map(|d| ImString::from(d.name().to_string()))
                            .collect::<Vec<_>>();

                        let building = buildings[self.cur_selected_building].to_str();
                        log::trace!("StartBuildingPlacement: {:?}", building);

                        let building_id = world
                            .fetch::<DefinitionStorage<core::defs::building::BuildingDefinition>>()
                            .find(building)
                            .unwrap()
                            .id()
                            .unwrap();

                        // How does this work? we should implement a blockaction in a global "input" resource
                        // so we can block pawn movements and de-select everything.
                        // Then we...fire a "BeginPlaceBuilding" event, which is handled by an input placement system?
                        // Then when its actually placed, a new system handles "Place building" actual
                        world
                            .fetch_mut::<EventChannel<PlayerInputEvent>>()
                            .single_write(PlayerInputEvent::StartBuildingPlacement { building_id });
                    }
                }
                if ui
                    .collapsing_header(im_str!("Tools"))
                    .default_open(true)
                    .build()
                {
                    let input_state = world.fetch::<InputState>();
                    if let Some(selection) = input_state.selection.as_ref() {
                        if let SelectionData::Building(entity) = selection {
                            use core::defs::reaction::Kind;

                            let (building_defs, reaction_defs, building_storage, _) =
                                <(
                                    ReadExpect<'_, DefinitionStorage<BuildingDefinition>>,
                                    ReadExpect<'_, DefinitionStorage<ReactionDefinition>>,
                                    ReadStorage<'_, components::BuildingComponent>,
                                    ReadStorage<'_, components::SpatialComponent>,
                                )>::fetch(world);

                            let building = building_storage.get(*entity).unwrap();
                            let def = building.fetch_def(&building_defs).unwrap();

                            let filter_location = |reagent: &Reagent| {
                                if let Kind::Location { name, .. } = &reagent.kind {
                                    if name.to_lowercase() == def.name().to_lowercase() {
                                        return true;
                                    }
                                }
                                false
                            };

                            // Create the reactions index for this building
                            // we have to search the reactions storage for all reactions which are AT this building
                            let valid_reactions = reaction_defs
                                .iter()
                                .filter_map(|reaction| {
                                    if reaction.reagents.iter().any(filter_location) {
                                        Some(ImString::from(reaction.name().to_string()))
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>();

                            if ui.button(im_str!("Add"), [0.0, 0.0]) {}
                            ui.same_line(0.0);
                            imgui::ComboBox::new(&ImString::from(format!(
                                "Start Recipe ##{}",
                                entity.id()
                            )))
                            .build_simple_string(
                                ui,
                                &mut self.selected_recipe,
                                &valid_reactions.iter().collect::<Vec<_>>(),
                            );
                        }
                    }
                }
            });
    }
}

#[derive(
    Shrinkwrap,
    Default,
    Copy,
    Clone,
    Debug,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct WindowId(usize);

pub struct Window {
    id: WindowId,
    inner: Box<dyn ImguiDrawable>,
    hidden: bool,
}

#[derive(Default)]
pub struct UiManager {
    windows: HashMap<&'static str, Window>,
}
impl UiManager {
    pub fn draw(&mut self, ui: &imgui::Ui, world: &mut World) {
        self.windows
            .iter_mut()
            .for_each(|(_, window)| window.inner.draw(ui, world));
    }

    pub fn add<W>(mut self, window: W, hidden: bool) -> Self
    where
        W: 'static + ImguiDrawable,
    {
        let id = WindowId(self.windows.len());
        self.windows.insert(
            window.name(),
            Window {
                inner: Box::new(window),
                hidden,
                id,
            },
        );

        self
    }

    pub fn build(mut self, world: &mut World) -> Self {
        self.windows
            .iter_mut()
            .for_each(|(_, window)| window.inner.setup(world));

        self
    }

    pub fn open(&mut self, name: &str) -> Result<(), failure::Error> {
        if let Some(window) = self.windows.get_mut(name) {
            window.hidden = window.inner.on_toggle_hidden(false);
            Ok(())
        } else {
            Err(failure::format_err!("Invalid window"))
        }
    }

    pub fn hide(&mut self, name: &str) -> Result<(), failure::Error> {
        if let Some(window) = self.windows.get_mut(name) {
            window.hidden = window.inner.on_toggle_hidden(true);
            Ok(())
        } else {
            Err(failure::format_err!("Invalid window"))
        }
    }
}
