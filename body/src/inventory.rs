use core::{
    amethyst::core::ecs::{storage::GenericReadStorage, Entity},
    components::ItemComponent,
    defs::{item::ItemDefinition, DefinitionComponent, DefinitionLookup},
    ItemHierarchy,
};

pub struct ChildItemIteratorEntry<'a> {
    pub entity: Entity,
    pub item: &'a ItemComponent,
    pub def: &'a ItemDefinition,
}

pub struct ChildItemIterator<'a, S, I>
where
    S: GenericReadStorage<Component = ItemComponent>,
    I: Iterator<Item = Entity>,
{
    index: usize,
    children: I,
    item_component_storage: &'a S,
    definition_storage: DefinitionLookup<'a, ItemDefinition>,
}
impl<'a, S, I> ChildItemIterator<'a, S, I>
where
    S: GenericReadStorage<Component = ItemComponent>,
    I: Iterator<Item = Entity>,
{
    pub fn new(
        parent: Entity,
        children: I,
        item_component_storage: &'a S,
        definition_storage: DefinitionLookup<'a, ItemDefinition>,
    ) -> Self {
        Self {
            index: 0,
            children,
            item_component_storage,
            definition_storage,
        }
    }
}
impl<'a, S, I> Iterator for ChildItemIterator<'a, S, I>
where
    S: GenericReadStorage<Component = ItemComponent>,
    I: Iterator<Item = Entity>,
{
    type Item = ChildItemIteratorEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entity) = self.children.next() {
            if let Some(item) = self.item_component_storage.get(entity) {
                self.index += 1;
                if let Some(def) = item.fetch_def(self.definition_storage.storage) {
                    return Some(ChildItemIteratorEntry { entity, item, def });
                }
            }
        }

        None
    }
}

pub fn get_all_items<'a, S>(
    parent: Entity,
    hierarchy: &'a ItemHierarchy,
    item_storage: &'a S,
    definition_storage: DefinitionLookup<'a, ItemDefinition>,
) -> impl Iterator<Item = ChildItemIteratorEntry<'a>>
where
    S: GenericReadStorage<Component = ItemComponent>,
{
    ChildItemIterator::new(
        parent,
        hierarchy.all_children_iter(parent),
        item_storage,
        definition_storage,
    )
}

pub fn get_all_items_filter<'a, F, S>(
    parent: Entity,
    hierarchy: &'a ItemHierarchy,
    item_storage: &'a S,
    definition_storage: DefinitionLookup<'a, ItemDefinition>,
    filter_function: F,
) -> impl Iterator<Item = ChildItemIteratorEntry<'a>>
where
    F: Fn(&Entity) -> bool,
    S: GenericReadStorage<Component = ItemComponent>,
{
    ChildItemIterator::new(
        parent,
        hierarchy.all_children_iter(parent).filter(filter_function),
        item_storage,
        definition_storage,
    )
}
