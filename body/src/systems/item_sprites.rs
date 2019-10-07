use core::{
    amethyst::{
        core::{shrev::ReaderId, SystemDesc},
        ecs::{
            storage::ComponentEvent, BitSet, Entities, Join, Read, ReadExpect, ReadStorage, System,
            SystemData, World, WriteStorage,
        },
    },
    components::{ItemComponent, ItemParentComponent, PropertiesComponent},
    defs::{item::ItemDefinition, sprites::SpriteSource, DefinitionComponent, DefinitionStorage},
    settings::GraphicsSettings,
    specs_hierarchy::HierarchyEvent,
    ItemHierarchy, SpriteRender,
};

// Does sanity checks on the inventory
pub struct ItemSpritesUpdateSystem {
    item_component_reader_id: ReaderId<ComponentEvent>,
    item_hierarchy_reader_id: ReaderId<HierarchyEvent>,
    new_items: BitSet,
}
impl<'s> System<'s> for ItemSpritesUpdateSystem {
    type SystemData = (
        Entities<'s>,
        ReadExpect<'s, GraphicsSettings>,
        ReadExpect<'s, ItemHierarchy>,
        Read<'s, DefinitionStorage<ItemDefinition>>,
        WriteStorage<'s, SpriteRender>,
        ReadStorage<'s, ItemComponent>,
        ReadStorage<'s, ItemParentComponent>,
        ReadStorage<'s, PropertiesComponent>,
    );

    fn run(
        &mut self,
        (
            entities,
            graphics_settings,
            hierarchy,
            item_defs,
            mut sprite_storage,
            item_storage,
            item_parents_storage,
            properties_storage,
        ): Self::SystemData,
    ) {
        for event in hierarchy.changed().read(&mut self.item_hierarchy_reader_id) {
            match event {
                HierarchyEvent::Modified(e) => {
                    if item_parents_storage.get(*e).is_some() {
                        // It was added as a child. Remove its sprite if it has it
                        log::trace!("Sprite is a child, removing sprite if it exists: {:?}", e);
                        sprite_storage.remove(*e);
                    }
                }
                HierarchyEvent::Removed(e) => {
                    if sprite_storage.get(*e).is_none() {
                        // No sprite currently added and the item was dropped.
                        log::warn!("Sprite replication for drops not implemented")
                    }
                }
            }
        }

        for event in item_storage
            .channel()
            .read(&mut self.item_component_reader_id)
        {
            if let ComponentEvent::Inserted(id) = event {
                self.new_items.add(*id);
            }
        }

        let mut new_sprites = Vec::new();

        for (entity, item, _, _, _) in (
            &entities,
            &item_storage,
            &self.new_items,
            !&sprite_storage,
            !&item_parents_storage,
        )
            .join()
        {
            // Item without sprite, that doesnt have a parent, that is new. We queue these for adding a sprite
            new_sprites.push((entity, &item.fetch_def(&item_defs).unwrap().sprite));
        }

        for (entity, item, _, _, _) in (
            &entities,
            &item_storage,
            &self.new_items,
            &sprite_storage,
            &item_parents_storage,
        )
            .join()
        {
            // Item with a sprite, WITH a parent.
            new_sprites.push((entity, &item.fetch_def(&item_defs).unwrap().sprite));
        }

        new_sprites.iter().for_each(|(entity, sprite)| {
            log::trace!("Adding sprite to orphaned item: {:?}", entity);

            let sheet_name = match &sprite.source {
                SpriteSource::Sheet(name) => name,
                _ => unimplemented!(),
            };

            sprite_storage
                .insert(
                    *entity,
                    SpriteRender {
                        sprite_sheet: (graphics_settings
                            .sprite_sheets
                            .get(sheet_name)
                            .expect("Invalid sheet name for sprite"))
                        .clone(),
                        sprite_number: sprite.index,
                        z_modifier: core::z_level_modifiers::ITEM,
                    },
                )
                .unwrap();
        });
    }
}

#[derive(Default)]
pub struct ItemSpritesUpdateSystemDesc;
impl<'a, 'b> SystemDesc<'a, 'b, ItemSpritesUpdateSystem> for ItemSpritesUpdateSystemDesc {
    fn build(self, world: &mut World) -> ItemSpritesUpdateSystem {
        log::trace!("Setup ItemSpritesUpdateSystem");
        <ItemSpritesUpdateSystem as System<'_>>::SystemData::setup(world);

        let item_hierarchy_reader_id = world.fetch_mut::<ItemHierarchy>().track();
        let item_component_reader_id =
            WriteStorage::<ItemComponent>::fetch(world).register_reader();
        ItemSpritesUpdateSystem {
            item_component_reader_id,
            item_hierarchy_reader_id,
            new_items: BitSet::new(),
        }
    }
}
