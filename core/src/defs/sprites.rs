use crate::{bitflags_serial, settings::GraphicsSettings, SpriteRender};
use amethyst::{
    ecs::{Builder, Entity, ReadExpect, SystemData, World, WriteStorage},
    renderer::{palette::Srgba, resources::Tint},
};
use bitflags::*;
use strum_macros::AsRefStr;

bitflags_serial! {
    pub struct SpriteOntoFlags: u8 {
        const All = 0;
        const SkipTint = 1;
    }
}

#[derive(
    AsRefStr,
    Debug,
    Clone,
    Hash,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum SpriteSource {
    Sheet(String),
    RexPaint(String),
}
impl Default for SpriteSource {
    fn default() -> Self {
        SpriteSource::Sheet("default_map".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SpriteRef {
    pub source: SpriteSource,
    pub index: usize,

    #[serde(default = "SpriteRef::default_scale")]
    pub scale: Option<f32>,

    #[serde(
        default = "SpriteRef::default_tint",
        with = "amethyst::renderer::serde_shim::srgba"
    )]
    pub tint: Srgba,
}
impl SpriteRef {
    fn default_tint() -> Srgba {
        Srgba::new(1.0, 1.0, 1.0, 1.0)
    }
    fn default_scale() -> Option<f32> {
        Some(1.0)
    }

    pub fn onto_builder<B>(
        &self,
        mut parent: B,
        z_modifier: f32,
        flags: SpriteOntoFlags,
        config: &GraphicsSettings,
    ) -> B
    where
        B: Builder,
    {
        match &self.source {
            SpriteSource::Sheet(name) => {
                if let Some(sheet) = config.sprite_sheets.get(name) {
                    parent = parent.with(SpriteRender {
                        sprite_sheet: sheet.clone(),
                        sprite_number: self.index,
                        z_modifier,
                    });
                    if !flags.contains(SpriteOntoFlags::SkipTint) {
                        parent = parent.with(Tint(self.tint));
                    }
                } else {
                    panic!("Invalid sheet specified")
                }
            }
            _ => unimplemented!(),
        };

        parent
    }

    pub fn onto_entity(
        &self,
        parent: Entity,
        world: &mut World,
        z_modifier: f32,
        flags: SpriteOntoFlags,
    ) {
        // TODO: support rexpaint, multiple-sprites, scaling.

        let (mut sprite_storage, mut tint_storage, config) = <(
            WriteStorage<'_, SpriteRender>,
            WriteStorage<'_, Tint>,
            ReadExpect<'_, GraphicsSettings>,
        )>::fetch(world);

        match &self.source {
            SpriteSource::Sheet(name) => {
                if let Some(sheet) = config.sprite_sheets.get(name) {
                    sprite_storage
                        .insert(
                            parent,
                            SpriteRender {
                                sprite_sheet: sheet.clone(),
                                sprite_number: self.index,
                                z_modifier,
                            },
                        )
                        .unwrap();
                    if !flags.contains(SpriteOntoFlags::SkipTint) {
                        tint_storage.insert(parent, Tint(self.tint)).unwrap();
                    }
                } else {
                    panic!("Invalid sheet specified")
                }
            }
            _ => unimplemented!(),
        }
    }
}
impl Default for SpriteRef {
    fn default() -> Self {
        Self {
            source: SpriteSource::default(),
            index: 18,
            tint: Self::default_tint(),
            scale: Self::default_scale(),
        }
    }
}

#[derive(
    AsRefStr,
    Debug,
    Clone,
    Hash,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum SpriteDefinition {
    Sheet(String),
    Tiled(String),
    RexPaint(String),
    None,
}
impl Default for SpriteDefinition {
    fn default() -> Self {
        Self::None
    }
}
