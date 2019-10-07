#![allow(clippy::default_trait_access, clippy::use_self)]

use amethyst::{
    assets::AssetStorage,
    core::{
        ecs::{
            prelude::DispatcherBuilder, Entities, Entity, Join, Read, ReadExpect, ReadStorage,
            SystemData, World, WorldExt,
        },
        math::Vector3,
        transform::Transform,
        Hidden, HiddenPropagate,
    },
    error::Error,
    renderer::{
        batch::{GroupIterator, OneLevelBatch, OrderedOneLevelBatch},
        bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
        pipeline::{PipelineDescBuilder, PipelinesBuilder},
        pod::SpriteArgs,
        rendy::{
            command::{QueueId, RenderPassEncoder},
            factory::Factory,
            graph::{
                render::{PrepareResult, RenderGroup, RenderGroupDesc},
                GraphContext, NodeBuffer, NodeImage,
            },
            hal::{self, device::Device, pso},
            mesh::AsVertex,
        },
        resources::Tint,
        sprite::SpriteSheet,
        sprite_visibility::{SpriteVisibility, SpriteVisibilitySortingSystem},
        submodules::{DynamicVertexBuffer, FlatEnvironmentSub, TextureId, TextureSub},
        types::{Backend, Texture},
        util,
    },
    tiles::{Map, TileMap},
};
use core::{
    components::SpatialComponent,
    derivative::Derivative,
    tiles::{region::RegionTile, CurrentTileZ},
    SpriteRender,
};

/// Draw opaque sprites without lighting.
#[derive(Derivative)]
#[derivative(Default(bound = ""), Debug(bound = ""))]
pub struct DrawSpritesDesc {}

impl DrawSpritesDesc {
    /// Create instance of `DrawSprites` render group
    pub fn new() -> Self {
        Self {}
    }
}

impl<B: Backend> RenderGroupDesc<B, World> for DrawSpritesDesc {
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &World,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, World>>, failure::Error> {
        let env = FlatEnvironmentSub::new(factory)?;
        let textures = TextureSub::new(factory)?;
        let vertex = DynamicVertexBuffer::new();

        let (pipeline, pipeline_layout) = build_sprite_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            false,
            vec![env.raw_layout(), textures.raw_layout()],
        )?;

        Ok(Box::new(DrawSprites::<B> {
            pipeline,
            pipeline_layout,
            env,
            textures,
            vertex,
            sprites: Default::default(),
        }))
    }
}

/// Draws opaque 2D sprites to the screen without lighting.
#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub struct DrawSprites<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: FlatEnvironmentSub<B>,
    textures: TextureSub<B>,
    vertex: DynamicVertexBuffer<B, SpriteArgs>,
    sprites: OneLevelBatch<TextureId, SpriteArgs>,
}

impl<B: Backend> RenderGroup<B, World> for DrawSprites<B> {
    #[allow(clippy::cast_precision_loss)]
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        world: &World,
    ) -> PrepareResult {
        let (
            entities,
            current_tile_z,
            sprite_sheet_storage,
            tex_storage,
            visibility,
            _,
            _,
            sprite_renders,
            transforms,
            tints,
            spatial_storage,
            maps_storage,
        ) = <(
            Entities<'_>,
            Read<'_, CurrentTileZ>,
            Read<'_, AssetStorage<SpriteSheet>>,
            Read<'_, AssetStorage<Texture>>,
            ReadExpect<'_, SpriteVisibility>,
            ReadStorage<'_, Hidden>,
            ReadStorage<'_, HiddenPropagate>,
            ReadStorage<'_, SpriteRender>,
            ReadStorage<'_, Transform>,
            ReadStorage<'_, Tint>,
            ReadStorage<'_, SpatialComponent>,
            ReadStorage<'_, TileMap<RegionTile>>,
        )>::fetch(world);

        let tile_dimensions = if let Some(map) = (&maps_storage).join().next() {
            Some(Vector3::new(
                map.tile_dimensions().x as f32,
                map.tile_dimensions().y as f32,
                map.tile_dimensions().z as f32,
            ))
        } else {
            None
        };

        self.env.process(factory, index, world);

        let sprites_ref = &mut self.sprites;
        let textures_ref = &mut self.textures;

        sprites_ref.clear_inner();

        {
            (
                &entities,
                &sprite_renders,
                &transforms,
                spatial_storage.maybe(),
                tints.maybe(),
                &visibility.visible_unordered,
            )
                .join()
                .filter_map(|(_, sprite_render, global, spatial, tint, _)| {
                    if global.translation().z.floor() < (current_tile_z.1).0
                        || global.translation().z.floor() > (current_tile_z.1).1
                    {
                        return None;
                    }
                    let (batch_data, texture) = SpriteRender::raw(
                        &tex_storage,
                        &sprite_sheet_storage,
                        &sprite_render,
                        &global,
                        tint,
                        spatial,
                        tile_dimensions,
                    )?;

                    let (tex_id, _) = textures_ref.insert(
                        factory,
                        world,
                        texture,
                        hal::image::Layout::ShaderReadOnlyOptimal,
                    )?;

                    Some((tex_id, batch_data))
                })
                .for_each_group(|tex_id, batch_data| {
                    sprites_ref.insert(tex_id, batch_data.drain(..))
                });
        }

        self.textures.maintain(factory, world);

        {
            sprites_ref.prune();
            self.vertex.write(
                factory,
                index,
                self.sprites.count() as u64,
                self.sprites.data(),
            );
        }

        PrepareResult::DrawRecord
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _world: &World,
    ) {
        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, layout, 0, &mut encoder);
        self.vertex.bind(index, 0, 0, &mut encoder);
        for (&tex, range) in self.sprites.iter() {
            if self.textures.loaded(tex) {
                self.textures.bind(layout, 1, tex, &mut encoder);
                unsafe {
                    encoder.draw(0..4, range);
                }
            }
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _world: &World) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}
/// Describes drawing transparent sprites without lighting.
#[derive(Derivative)]
#[derivative(Default(bound = ""), Debug(bound = ""))]
pub struct DrawSpritesTransparentDesc;
impl DrawSpritesTransparentDesc {
    /// Create instance of `DrawSprites` render group
    pub fn new() -> Self {
        Self {}
    }
}

impl<B: Backend> RenderGroupDesc<B, World> for DrawSpritesTransparentDesc {
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _world: &World,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, World>>, failure::Error> {
        let env = FlatEnvironmentSub::new(factory)?;
        let textures = TextureSub::new(factory)?;
        let vertex = DynamicVertexBuffer::new();

        let (pipeline, pipeline_layout) = build_sprite_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            true,
            vec![env.raw_layout(), textures.raw_layout()],
        )?;

        Ok(Box::new(DrawSpritesTransparent::<B> {
            pipeline,
            pipeline_layout,
            env,
            textures,
            vertex,
            sprites: Default::default(),
            change: Default::default(),
        }))
    }
}

/// Draws transparent sprites without lighting.
#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub struct DrawSpritesTransparent<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: FlatEnvironmentSub<B>,
    textures: TextureSub<B>,
    vertex: DynamicVertexBuffer<B, SpriteArgs>,
    sprites: OrderedOneLevelBatch<TextureId, SpriteArgs>,
    change: util::ChangeDetection,
}

impl<B: Backend> RenderGroup<B, World> for DrawSpritesTransparent<B> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        world: &World,
    ) -> PrepareResult {
        let (
            entities,
            current_tile_z,
            sprite_sheet_storage,
            tex_storage,
            visibility,
            sprite_renders,
            transforms,
            tints,
            spatial_storage,
            maps_storage,
        ) = <(
            Entities<'_>,
            Read<'_, CurrentTileZ>,
            Read<'_, AssetStorage<SpriteSheet>>,
            Read<'_, AssetStorage<Texture>>,
            ReadExpect<'_, SpriteVisibility>,
            ReadStorage<'_, SpriteRender>,
            ReadStorage<'_, Transform>,
            ReadStorage<'_, Tint>,
            ReadStorage<'_, SpatialComponent>,
            ReadStorage<'_, TileMap<RegionTile>>,
        )>::fetch(world);

        self.env.process(factory, index, world);
        self.sprites.swap_clear();
        let mut changed = false;

        let tile_dimensions = if let Some(map) = (&maps_storage).join().next() {
            Some(Vector3::new(
                map.tile_dimensions().x as f32,
                map.tile_dimensions().y as f32,
                map.tile_dimensions().z as f32,
            ))
        } else {
            None
        };

        let sprites_ref = &mut self.sprites;
        let textures_ref = &mut self.textures;
        {
            let mut joined = (
                &entities,
                &sprite_renders,
                &transforms,
                spatial_storage.maybe(),
                tints.maybe(),
            )
                .join();
            visibility
                .visible_ordered
                .iter()
                .filter_map(|e| joined.get_unchecked(e.id()))
                .filter_map(|(_, sprite_render, global, spatial, tint)| {
                    if global.translation().z.floor() < (current_tile_z.1).0
                        || global.translation().z.floor() > (current_tile_z.1).1
                    {
                        return None;
                    }

                    let (batch_data, texture) = SpriteRender::raw(
                        &tex_storage,
                        &sprite_sheet_storage,
                        &sprite_render,
                        &global,
                        tint,
                        spatial,
                        tile_dimensions,
                    )?;

                    let (tex_id, this_changed) = textures_ref.insert(
                        factory,
                        world,
                        texture,
                        hal::image::Layout::ShaderReadOnlyOptimal,
                    )?;

                    changed = changed || this_changed;
                    Some((tex_id, batch_data))
                })
                .for_each_group(|tex_id, batch_data| {
                    sprites_ref.insert(tex_id, batch_data.drain(..));
                });
        }
        self.textures.maintain(factory, world);
        changed = changed || self.sprites.changed();

        {
            self.vertex.write(
                factory,
                index,
                self.sprites.count() as u64,
                Some(self.sprites.data()),
            );
        }

        self.change.prepare_result(index, changed)
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _world: &World,
    ) {
        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, layout, 0, &mut encoder);
        self.vertex.bind(index, 0, 0, &mut encoder);
        for (&tex, range) in self.sprites.iter() {
            if self.textures.loaded(tex) {
                self.textures.bind(layout, 1, tex, &mut encoder);
                unsafe {
                    encoder.draw(0..4, range);
                }
            }
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &World) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}

fn build_sprite_pipeline<B: Backend>(
    factory: &Factory<B>,
    subpass: hal::pass::Subpass<'_, B>,
    framebuffer_width: u32,
    framebuffer_height: u32,
    transparent: bool,
    layouts: Vec<&B::DescriptorSetLayout>,
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), failure::Error> {
    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
    }?;

    let mut shader_set = super::SPRITES_SHADERSET.build(factory, Default::default())?;

    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                .with_vertex_desc(&[(SpriteArgs::vertex(), pso::VertexInputRate::Instance(1))])
                .with_input_assembler(pso::InputAssemblerDesc::new(hal::Primitive::TriangleStrip))
                .with_shaders(shader_set.raw()?)
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                .with_blend_targets(vec![pso::ColorBlendDesc(
                    pso::ColorMask::ALL,
                    if transparent {
                        pso::BlendState::PREMULTIPLIED_ALPHA
                    } else {
                        pso::BlendState::Off
                    },
                )])
                .with_depth_test(pso::DepthTest::On {
                    fun: pso::Comparison::Less,
                    write: !transparent,
                }),
        )
        .build(factory, None);

    shader_set.dispose(factory);

    match pipes {
        Err(e) => {
            unsafe {
                factory.device().destroy_pipeline_layout(pipeline_layout);
            }
            Err(e)
        }
        Ok(mut pipes) => Ok((pipes.remove(0), pipeline_layout)),
    }
}

// A [RenderPlugin] for drawing 2d objects with flat shading.
/// Required to display sprites defined with `SpriteRender` component.
#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub struct RenderSprites {
    target: Target,
    #[derivative(Debug = "ignore")]
    for_each_transparent_sprite:
        Option<Vec<Box<dyn Fn(Entity, &World, &mut SpriteArgs) + Send + Sync>>>,
    #[derivative(Debug = "ignore")]
    for_each_opaque_sprite: Option<Vec<Box<dyn Fn(Entity, &World, &mut SpriteArgs) + Send + Sync>>>,
}
impl Default for RenderSprites {
    fn default() -> Self {
        Self {
            target: Target::default(),
            for_each_transparent_sprite: Some(Vec::new()),
            for_each_opaque_sprite: Some(Vec::new()),
        }
    }
}

impl RenderSprites {
    /// Set target to which 2d sprites will be rendered.
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }
    pub fn with_for_each_opaque_sprite(
        mut self,
        closure: &'static (dyn Fn(Entity, &World, &mut SpriteArgs) + Send + Sync),
    ) -> Self {
        self.for_each_opaque_sprite
            .as_mut()
            .unwrap()
            .push(Box::new(closure));
        self
    }
    pub fn with_for_each_transparent_sprite(
        mut self,
        closure: &'static (dyn Fn(Entity, &World, &mut SpriteArgs) + Send + Sync),
    ) -> Self {
        self.for_each_transparent_sprite
            .as_mut()
            .unwrap()
            .push(Box::new(closure));
        self
    }
}

impl<B: Backend> RenderPlugin<B> for RenderSprites {
    fn on_build<'a, 'b>(
        &mut self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        world.register::<SpriteRender>();

        builder.add(
            SpriteVisibilitySortingSystem::new(),
            "sprite_visibility_system",
            &[],
        );
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        plan.extend_target(self.target, |ctx| {
            ctx.add(RenderOrder::Opaque, DrawSpritesDesc::new().builder())?;
            ctx.add(
                RenderOrder::Transparent,
                DrawSpritesTransparentDesc::new().builder(),
            )?;
            Ok(())
        });
        Ok(())
    }
}
