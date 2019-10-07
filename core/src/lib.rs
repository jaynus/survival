#![deny(clippy::pedantic, clippy::all)]
#![allow(
    clippy::type_complexity,
    clippy::empty_enum,
    clippy::default_trait_access,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::similar_names,
    clippy::use_self,
    clippy::module_name_repetitions
)]
#![feature(custom_attribute, core_intrinsics, const_fn)]
#![allow(dead_code, non_upper_case_globals, unused_attributes)]

pub mod defs;
pub mod settings;
#[macro_use]
pub mod bitflags_serial;
pub mod clock;
pub mod components;
pub mod embark;
pub mod fsm;
pub mod input;
pub mod utils;

pub mod initializers;

pub type ItemHierarchy = specs_hierarchy::Hierarchy<components::ItemParentComponent>;

pub use amethyst;
pub use arr_macro;
pub use bitflags;
pub use bitflags::*;
pub use derivative;
pub use failure;
pub use fern;
pub use fnv;
pub use hibitset;
pub use image;
pub use imageproc;
pub use indexmap;
pub use itertools;
pub use num_derive;
pub use num_traits;
pub use petgraph;
pub use rand;
pub use rand_distr;
pub use rand_xorshift;
pub use rayon;
pub use ron;
pub use serde;
pub use shrinkwraprs;
pub use slice_deque;
pub use smallvec;
pub use specs_hierarchy;
pub use spmc;
pub use strum;
pub use strum_macros;

pub type Error = failure::Error;
pub type Result<T> = std::result::Result<T, Error>;

mod component_event_reader;
pub use component_event_reader::*;

pub mod tiles;

pub mod tests {
    pub fn init_test_log() {
        let _ = env_logger::Builder::from_env(
            env_logger::Env::default()
                .filter_or("SURVIVAL_LOG_LEVEL", "trace")
                .write_style_or("SURVIVAL_LOG_STYLE", "always"),
        )
        .is_test(true)
        .try_init();
    }
}

#[cfg(feature = "nightly")]
pub use core::format_args;

pub mod z_level_modifiers {
    pub const TOP: f32 = 0.8;
    pub const TILE: f32 = 0.0;
    pub const PAWN: f32 = 0.6;
    pub const ITEM: f32 = 0.5;
    pub const BUILDING: f32 = 0.2;
    pub const FOLIAGE: f32 = 0.1;
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpriteRender {
    pub sprite_sheet: amethyst::assets::Handle<amethyst::renderer::SpriteSheet>,

    pub sprite_number: usize,

    pub z_modifier: f32,
}
impl amethyst::ecs::Component for SpriteRender {
    type Storage = amethyst::ecs::VecStorage<Self>;
}
impl SpriteRender {
    pub fn raw<'a>(
        tex_storage: &amethyst::assets::AssetStorage<amethyst::renderer::Texture>,
        sprite_storage: &'a amethyst::assets::AssetStorage<amethyst::renderer::SpriteSheet>,
        sprite_render: &Self,
        transform: &amethyst::core::Transform,
        tint: Option<&amethyst::renderer::resources::Tint>,
        _: Option<&components::SpatialComponent>,
        tile_dimensions: Option<amethyst::core::math::Vector3<f32>>,
    ) -> Option<(
        amethyst::renderer::pod::SpriteArgs,
        &'a amethyst::assets::Handle<amethyst::renderer::Texture>,
    )> {
        use amethyst::renderer::pod::IntoPod;

        let sprite_sheet = sprite_storage.get(&sprite_render.sprite_sheet)?;
        if !tex_storage.contains(&sprite_sheet.texture) {
            return None;
        }

        let sprite = &sprite_sheet.sprites[sprite_render.sprite_number];

        let mut transform = amethyst::core::math::convert::<_, amethyst::core::math::Matrix4<f32>>(
            *transform.global_matrix(),
        );
        transform.column_mut(3)[2] += sprite_render.z_modifier;

        if let Some(tile_dimensions) = tile_dimensions {
            if sprite.height > tile_dimensions.x {
                transform.column_mut(3)[0] +=
                    ((sprite.width / tile_dimensions.x) - 1.0) * (tile_dimensions.x * 0.5)
            }

            if sprite.height > tile_dimensions.y {
                transform.column_mut(3)[1] -=
                    ((sprite.height / tile_dimensions.y) - 1.0) * (tile_dimensions.y * 0.5)
            }
        }

        let dir_x = transform.column(0) * sprite.width;
        let dir_y = transform.column(1) * -sprite.height;
        let pos = transform
            * amethyst::core::math::Vector4::new(-sprite.offsets[0], -sprite.offsets[1], 0.0, 1.0);

        Some((
            amethyst::renderer::pod::SpriteArgs {
                dir_x: dir_x.xy().into_pod(),
                dir_y: dir_y.xy().into_pod(),
                pos: pos.xy().into_pod(),
                u_offset: [sprite.tex_coords.left, sprite.tex_coords.right].into(),
                v_offset: [sprite.tex_coords.top, sprite.tex_coords.bottom].into(),
                depth: pos.z,
                tint: tint.map_or([1.0; 4].into(), |t| {
                    let (r, g, b, a) = t.0.into_components();
                    [r, g, b, a].into()
                }),
            },
            &sprite_sheet.texture,
        ))
    }
}
