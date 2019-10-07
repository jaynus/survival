use amethyst::{
    core::{components::Transform, math::Point3},
    ecs::{
        Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, SystemData, World, Write,
        WriteStorage,
    },
    shrev::EventChannel,
    tiles::{Map, TileMap},
};

use crate::components::{
    AttributesComponent, CurrentActionComponent, IdleComponent, ItemComponent, ItemParentComponent,
    ItemParentRelationship, PawnComponent, PyscheNeedsComponent, RaceComponent,
};
use amethyst_imgui::imgui::{self, im_str, Condition, ImString, Ui};
use core::{
    defs::{
        action::ActionDefinition, item::ItemDefinition, DefinitionLookup, DefinitionStorage, Named,
    },
    fsm::{ActionEvent, Event, MovementEvent},
    rand::{thread_rng, Rng},
    ItemHierarchy,
};
use std::collections::HashMap;

type PawnData<'a> = (
    Read<'a, LazyUpdate>,
    Entities<'a>,
    ReadExpect<'a, DefinitionStorage<ActionDefinition>>,
    WriteStorage<'a, PawnComponent>,
    WriteStorage<'a, IdleComponent>,
    ReadStorage<'a, CurrentActionComponent>,
    ReadStorage<'a, ItemComponent>,
    ReadStorage<'a, Transform>,
    ReadStorage<'a, TileMap<core::tiles::region::RegionTile>>,
    ReadStorage<'a, RaceComponent>,
    ReadStorage<'a, PyscheNeedsComponent>,
    ReadStorage<'a, AttributesComponent>,
    Write<'a, EventChannel<ActionEvent>>,
);

#[derive(Default)]
struct PawnDebugWindowStateEntry {
    selected_item: usize,
    selected_behavior: usize,
}
type PawnDebugWindowState = HashMap<u32, PawnDebugWindowStateEntry>;

pub fn setup(world: &mut World) {
    PawnData::setup(world);
    <(Write<'_, PawnDebugWindowState>)>::setup(world);
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
pub fn draw(ui: &Ui, world: &mut World) {
    imgui::Window::new(im_str!("Pawn##Debug"))
        .size([500.0, 1000.0], Condition::FirstUseEver)
        .build(ui, || {
            let (
                lazy,
                entities,
                _action_storage,
                mut pawns,
                _idles,
                active_action_storage,
                items_storage,
                transform_storage,
                tilemap_storage,
                _race_storage,
                needs_storage,
                _attributes_storage,
                mut action_channel,
            ) = PawnData::fetch(&world);

            let mut window_state = <(Write<'_, PawnDebugWindowState>)>::fetch(&world);

            for (entity, _) in (&*entities, &mut pawns).join() {
                let pawn_name = format!("pawn {}", entity.id());

                if !window_state.contains_key(&entity.id()) {
                    window_state.insert(entity.id(), PawnDebugWindowStateEntry::default());
                }

                if ui
                    .collapsing_header(&ImString::from(pawn_name))
                    .default_open(true)
                    .build()
                {
                    let tile_map = (&tilemap_storage).join().next().unwrap();
                    if let Some(transform) = transform_storage.get(entity) {
                        if let Some(coord) = tile_map.to_tile(transform.translation()) {
                            ui.text(&format!("loc: ({}, {}, {})", coord.x, coord.y, coord.z));
                        } else {
                            panic!("INVALID COORDINATE: {:?}", transform.translation())
                        }
                    }
                    ui.group(|| {
                        use core::defs::psyche::NeedKind;
                        use strum::IntoEnumIterator;
                        if let Some(needs) = needs_storage.get(entity) {
                            for kind in NeedKind::iter() {
                                let name: &str = kind.as_ref();
                                ui.text(&format!("{:?} = {:?}", name, needs.need(kind).value));
                            }
                        }
                    });
                    ui.group(|| {
                        let items = world
                            .fetch::<DefinitionStorage<ItemDefinition>>()
                            .iter()
                            .map(|d| ImString::from(d.name().to_string()))
                            .collect::<Vec<_>>();
                        //items.sort();

                        // List items the pawn has
                        {
                            let item_hierarchy = world.fetch::<ItemHierarchy>();
                            let defs = world.fetch::<DefinitionStorage<ItemDefinition>>();

                            let items = body::inventory::get_all_items(
                                entity,
                                &*item_hierarchy,
                                &items_storage,
                                DefinitionLookup::new(&*defs),
                            );

                            ui.columns(
                                4,
                                &ImString::from(format!("Items ##{}", entity.id())),
                                true,
                            );
                            for item in items {
                                ui.text(item.def.name());
                                ui.next_column();
                            }
                        }

                        ui.columns(1, im_str!(""), false);

                        imgui::ComboBox::new(&ImString::from(format!(
                            "Add Item ##{}",
                            entity.id()
                        )))
                        .build_simple_string(
                            ui,
                            &mut window_state.get_mut(&entity.id()).unwrap().selected_item,
                            &items.iter().collect::<Vec<_>>(),
                        );

                        if ui.button(
                            &ImString::from(format!("Spawn ##{}", entity.id())),
                            [0.0, 0.0],
                        ) {
                            log::debug!("Spawn item on pawn! id={}", entity.id());
                            let entity_clone = entity;
                            let selected_item = window_state[&entity.id()].selected_item;
                            let name = items[selected_item].to_str().to_string();

                            lazy.exec_mut(move |lazy_world| {
                                crate::initializers::spawn_item(
                                    name.as_str(),
                                    None,
                                    None,
                                    Some(ItemParentComponent::new(
                                        entity_clone,
                                        ItemParentRelationship::Worn,
                                    )),
                                    None,
                                    lazy_world,
                                );
                            });
                        }
                    });
                    ui.group(|| {
                        if active_action_storage.contains(entity) {
                            let task_text = format!(
                                "Current Action: {}",
                                active_action_storage
                                    .get(entity)
                                    .unwrap()
                                    .inner
                                    .event
                                    .to_string()
                            );
                            ui.text(&ImString::from(task_text));
                        } else {
                            ui.text(im_str!("Current Action: None"));
                        }

                        if ui.button(
                            &ImString::from(format!("Random Movement Event {}", entity.id())),
                            [0.0, 0.0],
                        ) {
                            let mut rng = thread_rng();
                            let location = Point3::new(
                                rng.gen_range(0, tile_map.dimensions().x),
                                rng.gen_range(0, tile_map.dimensions().y),
                                0,
                            );

                            action_channel.single_write(ActionEvent::new(
                                Some(entity),
                                None,
                                None,
                                Event::Move(MovementEvent::To(location)),
                            ));
                        }
                    });
                }

                ui.separator();
            }
        });
}
