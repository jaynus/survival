pub mod sprites;

use amethyst::renderer::rendy::shader::{
    ShaderKind, ShaderSetBuilder, SourceLanguage, SourceShaderInfo, SpirvReflection, SpirvShader,
};

pub use sprites::RenderSprites;

lazy_static::lazy_static! {
    pub static ref SPRITES_VERTEX: SpirvShader = SourceShaderInfo::new(
        include_str!("shaders/sprites.vert"),
        "shaders/sprites.vert",
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    pub static ref SPRITES_FRAGMENT: SpirvShader = SourceShaderInfo::new(
        include_str!("shaders/sprites.frag"),
        "shaders/sprites.frag",
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    pub static ref SPRITES_SHADERSET: ShaderSetBuilder = ShaderSetBuilder::default()
        .with_vertex(&*SPRITES_VERTEX).unwrap()
        .with_fragment(&*SPRITES_FRAGMENT).unwrap();
}

lazy_static::lazy_static! {
    pub static ref SPRITES_REFLECTION: SpirvReflection = SPRITES_SHADERSET.reflect().unwrap();
}
