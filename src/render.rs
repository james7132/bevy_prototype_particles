use crate::{
    material::{ParticleMaterial, ParticleMaterialUniformData},
    particles::Particles,
};
use bevy::{
    ecs::system::SystemState,
    prelude::*,
    render2::{
        core_pipeline::Transparent3dPhase,
        mesh::{shape::Quad, Indices, Mesh, VertexAttributeValues},
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraphContext},
        render_phase::{Draw, DrawFunctions, Drawable, RenderPhase, TrackedRenderPass},
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderQueue},
        shader::Shader,
        texture::{BevyDefault, GpuImage, Image, TextureFormatPixelInfo},
        view::{ViewMeta, ViewUniform, ViewUniformOffset},
    },
    utils::slab::{FrameSlabMap, FrameSlabMapKey},
};
use bytemuck::{Pod, Zeroable};
use crevice::std140::AsStd140;
use std::{num::NonZeroU64, ops::Range};
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindingResource, BufferBinding,
    BufferUsage, Face, FragmentState, FrontFace, ImageCopyTexture, ImageDataLayout,
    MultisampleState, Origin3d, PolygonMode, PrimitiveState, PrimitiveTopology,
};
use wgpu_types::IndexFormat;

pub struct ParticleShaders {
    pipeline: RenderPipeline,

    view_layout: BindGroupLayout,
    particle_layout: BindGroupLayout,
    material_layout: BindGroupLayout,

    positions: BufferVec<Vec4>,
    sizes: BufferVec<f32>,
    colors: BufferVec<Vec4>,

    // This dummy white texture is to be used in place of optional StandardMaterial textures
    dummy_white_gpu_image: GpuImage,
}

impl FromWorld for ParticleShaders {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();
        let shader = Shader::from_wgsl(include_str!("particle.wgsl"));
        let shader_module = render_device.create_shader_module(&shader);

        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: BufferSize::new(std::mem::size_of::<ViewUniform>() as u64),
                },
                count: None,
            }],
            label: None,
        });

        let particle_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // Positions/Rotations
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(std::mem::size_of::<Vec4>() as u64),
                    },
                    count: None,
                },
                // Sizes
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(std::mem::size_of::<f32>() as u64),
                    },
                    count: None,
                },
                // Colors
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(std::mem::size_of::<Vec4>() as u64),
                    },
                    count: None,
                },
            ],
        });

        let material_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(
                            ParticleMaterialUniformData::std140_size_static() as u64,
                        ),
                    },
                    count: None,
                },
                // Base Color Texture
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Base Color Texture Sampler
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler {
                        comparison: false,
                        filtering: true,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = render_device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[&view_layout, &particle_layout, &material_layout],
        });

        let pipeline = render_device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            vertex: VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent {
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::One,
                            operation: BlendOperation::Add,
                        },
                    }),
                    write_mask: ColorWrite::ALL,
                }],
            }),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Less,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            layout: Some(&pipeline_layout),
            multisample: MultisampleState::default(),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
        });

        // A 1x1x1 'all 1.0' texture to use as a dummy texture to use in place of optional StandardMaterial textures
        let dummy_white_gpu_image = {
            let image = Image::new_fill(
                Extent3d::default(),
                TextureDimension::D2,
                &[255u8; 4],
                TextureFormat::bevy_default(),
            );
            let texture = render_device.create_texture(&image.texture_descriptor);
            let sampler = render_device.create_sampler(&image.sampler_descriptor);

            let format_size = image.texture_descriptor.format.pixel_size();
            let render_queue = world.get_resource_mut::<RenderQueue>().unwrap();
            render_queue.write_texture(
                ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                },
                &image.data,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        std::num::NonZeroU32::new(
                            image.texture_descriptor.size.width * format_size as u32,
                        )
                        .unwrap(),
                    ),
                    rows_per_image: None,
                },
                image.texture_descriptor.size,
            );

            let texture_view = texture.create_view(&TextureViewDescriptor::default());
            GpuImage {
                texture,
                texture_view,
                sampler,
            }
        };

        Self {
            pipeline,

            view_layout,
            particle_layout,
            material_layout,

            positions: BufferVec::new(BufferUsage::STORAGE),
            sizes: BufferVec::new(BufferUsage::STORAGE),
            colors: BufferVec::new(BufferUsage::STORAGE),

            dummy_white_gpu_image,
        }
    }
}

pub struct ExtractedParticle {
    material: Handle<ParticleMaterial>,
    positions: Vec<Vec4>,
    sizes: Vec<f32>,
    colors: Vec<Vec4>,
}

pub struct ExtractedParticles {
    particles: Vec<ExtractedParticle>,
}

pub fn extract_particles(
    mut commands: Commands,
    materials: Res<Assets<ParticleMaterial>>,
    images: Res<Assets<Image>>,
    query: Query<(&Particles, &Handle<ParticleMaterial>)>,
) {
    let mut extracted_particles = Vec::new();
    for (particles, material_handle) in query.iter() {
        if let Some(ref material) = materials.get(material_handle) {
            if let Some(ref image) = material.base_color_texture {
                if !images.contains(image) {
                    continue;
                }
            }

            // TODO(james7132): Find a way to do this without
            extracted_particles.push(ExtractedParticle {
                material: material_handle.clone_weak(),
                positions: particles.positions.clone(),
                sizes: particles.sizes.clone(),
                colors: particles.colors.clone(),
            });
        }
    }

    commands.insert_resource(ExtractedParticles {
        particles: extracted_particles,
    });
}

#[derive(Default)]
pub struct ParticleMeta {
    ranges: Vec<Range<u64>>,
    total_count: u64,
    view_bind_group: Option<BindGroup>,
    particle_bind_group: Option<BindGroup>,

    material_bind_groups: FrameSlabMap<Handle<ParticleMaterial>, BindGroup>,
    material_bind_group_keys: Vec<FrameSlabMapKey<Handle<ParticleMaterial>, BindGroup>>,
}

pub fn prepare_particles(
    render_device: Res<RenderDevice>,
    mut particle_meta: ResMut<ParticleMeta>,
    mut particle_shaders: ResMut<ParticleShaders>,
    extracted_particles: Res<ExtractedParticles>,
) {
    let mut total_count = 0;
    for particle in extracted_particles.particles.iter() {
        total_count += particle.positions.len();
    }

    particle_meta.total_count = total_count as u64;
    particle_meta.ranges.clear();
    if total_count == 0 {
        return;
    }

    particle_shaders
        .positions
        .reserve_and_clear(total_count, &render_device);
    particle_shaders
        .sizes
        .reserve_and_clear(total_count, &render_device);
    particle_shaders
        .colors
        .reserve_and_clear(total_count, &render_device);

    let mut start: u64 = 0;
    for particle in extracted_particles.particles.iter() {
        batch_copy(&particle.positions, &mut particle_shaders.positions);
        batch_copy(&particle.sizes, &mut particle_shaders.sizes);
        batch_copy(&particle.colors, &mut particle_shaders.colors);
        particle_meta
            .ranges
            .push(start..start + particle.positions.len() as u64);
        start += particle.positions.len() as u64;
    }

    particle_shaders
        .positions
        .write_to_staging_buffer(&render_device);
    particle_shaders
        .sizes
        .write_to_staging_buffer(&render_device);
    particle_shaders
        .colors
        .write_to_staging_buffer(&render_device);
}

fn batch_copy<T: Pod>(src: &Vec<T>, dst: &mut BufferVec<T>) {
    for item in src.iter() {
        dst.push(*item);
    }
}

fn bind_buffer<T: Pod>(buffer: &BufferVec<T>, count: u64) -> BindingResource {
    BindingResource::Buffer(BufferBinding {
        buffer: buffer.buffer().expect("missing buffer"),
        offset: 0,
        size: Some(NonZeroU64::new(std::mem::size_of::<T>() as u64 * count).unwrap()),
    })
}

fn image_handle_to_view_sampler<'a>(
    particle_shaders: &'a ParticleShaders,
    gpu_images: &'a RenderAssets<Image>,
    image_option: &Option<Handle<Image>>,
) -> (&'a TextureView, &'a Sampler) {
    image_option.as_ref().map_or(
        (
            &particle_shaders.dummy_white_gpu_image.texture_view,
            &particle_shaders.dummy_white_gpu_image.sampler,
        ),
        |image_handle| {
            let gpu_image = gpu_images
                .get(image_handle)
                .expect("only materials with valid textures should be drawn");
            (&gpu_image.texture_view, &gpu_image.sampler)
        },
    )
}

#[allow(clippy::too_many_arguments)]
pub fn queue_particles(
    draw_functions: Res<DrawFunctions>,
    render_device: Res<RenderDevice>,
    mut particle_meta: ResMut<ParticleMeta>,
    view_meta: Res<ViewMeta>,
    particle_shaders: Res<ParticleShaders>,
    extracted_particles: Res<ExtractedParticles>,
    render_materials: Res<RenderAssets<ParticleMaterial>>,
    gpu_images: Res<RenderAssets<Image>>,
    mut views: Query<&mut RenderPhase<Transparent3dPhase>>,
) {
    if view_meta.uniforms.is_empty() || particle_meta.total_count == 0 {
        return;
    }

    // TODO: define this without needing to check every frame
    particle_meta.view_bind_group.get_or_insert_with(|| {
        render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: view_meta.uniforms.binding(),
            }],
            label: None,
            layout: &particle_shaders.view_layout,
        })
    });

    // TODO(james7132): Find a way to cache this.
    particle_meta.particle_bind_group =
        Some(render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: bind_buffer(&particle_shaders.positions, particle_meta.total_count),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: bind_buffer(&particle_shaders.sizes, particle_meta.total_count),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: bind_buffer(&particle_shaders.colors, particle_meta.total_count),
                },
            ],
            label: None,
            layout: &particle_shaders.particle_layout,
        }));

    // let particle_meta = &mut *particle_meta;
    let draw_sprite_function = draw_functions.read().get_id::<DrawParticle>().unwrap();
    particle_meta.material_bind_groups.next_frame();
    particle_meta.material_bind_group_keys.clear();
    for mut transparent_phase in views.iter_mut() {
        for (i, particle) in extracted_particles.particles.iter().enumerate() {
            let gpu_material = render_materials
                .get(&particle.material)
                .expect("Failed to get ParticleMaterial PreparedAsset");

            let bind_group_key = particle_meta.material_bind_groups.get_or_insert_with(
                particle.material.clone_weak(),
                || {
                    let (base_color_texture_view, base_color_sampler) =
                        image_handle_to_view_sampler(
                            &particle_shaders,
                            &gpu_images,
                            &gpu_material.base_color_texture,
                        );

                    render_device.create_bind_group(&BindGroupDescriptor {
                        entries: &[
                            BindGroupEntry {
                                binding: 0,
                                resource: gpu_material.buffer.as_entire_binding(),
                            },
                            BindGroupEntry {
                                binding: 1,
                                resource: BindingResource::TextureView(&base_color_texture_view),
                            },
                            BindGroupEntry {
                                binding: 2,
                                resource: BindingResource::Sampler(&base_color_sampler),
                            },
                        ],
                        label: None,
                        layout: &particle_shaders.material_layout,
                    })
                },
            );
            particle_meta.material_bind_group_keys.push(bind_group_key);

            transparent_phase.add(Drawable {
                draw_function: draw_sprite_function,
                draw_key: i,
                sort_key: bind_group_key.index(),
            });
        }
    }
}

type DrawParticleQuery<'s, 'w> = (
    Res<'w, ParticleShaders>,
    Res<'w, ParticleMeta>,
    Query<'w, 's, &'w ViewUniformOffset>,
);

pub struct DrawParticle {
    params: SystemState<DrawParticleQuery<'static, 'static>>,
}

impl DrawParticle {
    pub fn new(world: &mut World) -> Self {
        Self {
            params: SystemState::new(world),
        }
    }
}

impl Draw for DrawParticle {
    fn draw<'w>(
        &mut self,
        world: &'w World,
        pass: &mut TrackedRenderPass<'w>,
        view: Entity,
        draw_key: usize,
        _sort_key: usize,
    ) {
        let (shaders, particle_meta, views) = self.params.get(world);
        let view_uniform = views.get(view).unwrap();
        let particle_meta = particle_meta.into_inner();

        if let Some(range) = particle_meta.ranges.get(draw_key).as_ref() {
            let vertex_range = (range.start * 6) as u32..(range.end * 6) as u32;

            pass.set_render_pipeline(&shaders.into_inner().pipeline);
            pass.set_bind_group(
                0,
                particle_meta.view_bind_group.as_ref().unwrap(),
                &[view_uniform.offset],
            );
            pass.set_bind_group(1, particle_meta.particle_bind_group.as_ref().unwrap(), &[]);
            pass.set_bind_group(
                2,
                &particle_meta.material_bind_groups
                    [particle_meta.material_bind_group_keys[draw_key]],
                &[],
            );
            pass.draw(vertex_range, 0..1);
        }
    }
}

pub struct ParticleNode;

impl Node for ParticleNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let particle_buffers = world.get_resource::<ParticleShaders>().unwrap();
        particle_buffers
            .positions
            .write_to_buffer(&mut render_context.command_encoder);
        particle_buffers
            .sizes
            .write_to_buffer(&mut render_context.command_encoder);
        particle_buffers
            .colors
            .write_to_buffer(&mut render_context.command_encoder);
        Ok(())
    }
}
