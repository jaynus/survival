#![allow(clippy::module_name_repetitions)]

use crate::systems::{item_sprites::ItemSpritesUpdateSystemDesc, BodyUpdatePropertiesSystemDesc};
use core::amethyst::core::{
    ecs::{prelude::DispatcherBuilder, World},
    SystemBundle, SystemDesc,
};

#[derive(Default)]
pub struct BodyBundle<'a> {
    dep: &'a [&'a str],
}

impl<'a> BodyBundle<'a> {
    /// Set dependencies for the `HierarchySystem<ItemParentComponent>`
    pub fn with_dep(mut self, dep: &'a [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c> SystemBundle<'a, 'b> for BodyBundle<'c> {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), core::amethyst::Error> {
        builder.add(
            core::specs_hierarchy::HierarchySystem::<core::components::ItemParentComponent>::new(
                world,
            ),
            "item_hierarchy_system",
            self.dep,
        );
        builder.add(
            BodyUpdatePropertiesSystemDesc::default().build(world),
            "body_update_system",
            &[],
        );
        builder.add(
            ItemSpritesUpdateSystemDesc::default().build(world),
            "item_sprite_update_system",
            &["item_hierarchy_system"],
        );
        Ok(())
    }
}
