use crate::particles::Particles;
use bevy::{
    ecs::system::SystemState,
    prelude::*,
    render2::{
        core_pipeline::Transparent3dPhase,
        mesh::{shape::Quad, Indices, Mesh, VertexAttributeValues},
        render_graph::{Node, NodeRunError, RenderGraphContext},
        render_phase::{Draw, DrawFunctions, Drawable, RenderPhase, TrackedRenderPass},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        shader::Shader,
        texture::BevyDefault,
        view::{ViewMeta, ViewUniform, ViewUniformOffset},
    },
};
use bytemuck::{Pod, Zeroable};
use std::ops::Range;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BufferUsage, Face,
    FragmentState, FrontFace, MultisampleState, PolygonMode, PrimitiveState, PrimitiveTopology,
};
use wgpu_types::IndexFormat;

pub struct ParticleMaterial {}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct ParticleVertex {
    position: Vec3,
    uv: Vec2,
}

pub struct ParticleShaders {
    pipeline: RenderPipeline,
    view_layout: BindGroupLayout,
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

        let pipeline_layout = render_device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[&view_layout],
        });

        let pipeline = render_device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            vertex: VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[
                    // Positions/Rotations
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vec4>() as wgpu::BufferAddress,
                        step_mode: wgpu::InputStepMode::Instance,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x4,
                        }],
                    },
                    // Sizes
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<f32>() as wgpu::BufferAddress,
                        step_mode: wgpu::InputStepMode::Instance,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32,
                        }],
                    },
                    // Colors
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vec4>() as wgpu::BufferAddress,
                        step_mode: wgpu::InputStepMode::Instance,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x4,
                        }],
                    },
                ],
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

        Self {
            pipeline,
            view_layout,
        }
    }
}

pub struct ExtractedParticle {
    positions: Vec<Vec4>,
    sizes: Vec<f32>,
    colors: Vec<Vec4>,
}

pub struct ExtractedParticles {
    particles: Vec<ExtractedParticle>,
}

pub fn extract_particles(mut commands: Commands, query: Query<(&Particles, &ParticleMaterial)>) {
    let mut extracted_particles = Vec::new();
    for (particles, _) in query.iter() {
        // TODO(james7132): Find a way to do this without
        extracted_particles.push(ExtractedParticle {
            positions: particles.positions.clone(),
            sizes: particles.sizes.clone(),
            colors: particles.colors.clone(),
        });
    }

    commands.insert_resource(ExtractedParticles {
        particles: extracted_particles,
    });
}

pub struct ParticleMeta {
    positions: BufferVec<Vec4>,
    sizes: BufferVec<f32>,
    colors: BufferVec<Vec4>,
    ranges: Vec<Range<u64>>,
    view_bind_group: Option<BindGroup>,
}

impl Default for ParticleMeta {
    fn default() -> Self {
        Self {
            positions: BufferVec::new(BufferUsage::VERTEX),
            sizes: BufferVec::new(BufferUsage::VERTEX),
            colors: BufferVec::new(BufferUsage::VERTEX),
            view_bind_group: None,
            ranges: Vec::new(),
        }
    }
}

pub fn prepare_particles(
    render_device: Res<RenderDevice>,
    mut particle_meta: ResMut<ParticleMeta>,
    extracted_particles: Res<ExtractedParticles>,
) {
    let mut total_count = 0;
    for particle in extracted_particles.particles.iter() {
        total_count += particle.positions.len();
    }

    particle_meta.ranges.clear();
    if total_count == 0 {
        return;
    }

    particle_meta
        .positions
        .reserve_and_clear(total_count, &render_device);
    particle_meta
        .sizes
        .reserve_and_clear(total_count, &render_device);
    particle_meta
        .colors
        .reserve_and_clear(total_count, &render_device);

    let mut start: u64 = 0;
    for particle in extracted_particles.particles.iter() {
        batch_copy(&particle.positions, &mut particle_meta.positions);
        batch_copy(&particle.sizes, &mut particle_meta.sizes);
        batch_copy(&particle.colors, &mut particle_meta.colors);
        particle_meta
            .ranges
            .push(start..start + particle.positions.len() as u64);
        start += particle.positions.len() as u64;
    }

    particle_meta
        .positions
        .write_to_staging_buffer(&render_device);
    particle_meta.sizes.write_to_staging_buffer(&render_device);
    particle_meta.colors.write_to_staging_buffer(&render_device);
}

fn batch_copy<T: Pod>(src: &Vec<T>, dst: &mut BufferVec<T>) {
    for item in src.iter() {
        dst.push(*item);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn queue_particles(
    draw_functions: Res<DrawFunctions>,
    render_device: Res<RenderDevice>,
    mut particle_meta: ResMut<ParticleMeta>,
    view_meta: Res<ViewMeta>,
    particle_shaders: Res<ParticleShaders>,
    extracted_particles: Res<ExtractedParticles>,
    // gpu_images: Res<RenderAssets<Image>>,
    mut views: Query<&mut RenderPhase<Transparent3dPhase>>,
) {
    if view_meta.uniforms.is_empty() || extracted_particles.particles.is_empty() {
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
    // let particle_meta = &mut *particle_meta;
    let draw_sprite_function = draw_functions.read().get_id::<DrawParticle>().unwrap();
    // sprite_meta.texture_bind_groups.next_frame();
    // sprite_meta.texture_bind_group_keys.clear();
    for mut transparent_phase in views.iter_mut() {
        for (i, _) in extracted_particles.particles.iter().enumerate() {
            // let texture_bind_group_key = sprite_meta.texture_bind_groups.get_or_insert_with(
            //     sprite.handle.clone_weak(),
            //     || {
            //         let gpu_image = gpu_images.get(&sprite.handle).unwrap();
            //         render_device.create_bind_group(&BindGroupDescriptor {
            //             entries: &[
            //                 BindGroupEntry {
            //                     binding: 0,
            //                     resource: BindingResource::TextureView(&gpu_image.texture_view),
            //                 },
            //                 BindGroupEntry {
            //                     binding: 1,
            //                     resource: BindingResource::Sampler(&gpu_image.sampler),
            //                 },
            //             ],
            //             label: None,
            //             layout: &sprite_shaders.material_layout,
            //         })
            //     },
            // );
            // sprite_meta
            //     .texture_bind_group_keys
            //     .push(texture_bind_group_key);

            transparent_phase.add(Drawable {
                draw_function: draw_sprite_function,
                draw_key: i,
                sort_key: i,
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
            pass.set_render_pipeline(&shaders.into_inner().pipeline);
            pass.set_bind_group(
                0,
                particle_meta.view_bind_group.as_ref().unwrap(),
                &[view_uniform.offset],
            );
            pass.set_vertex_buffer(0, particle_meta.positions.buffer().unwrap().slice(..));
            pass.set_vertex_buffer(1, particle_meta.sizes.buffer().unwrap().slice(..));
            pass.set_vertex_buffer(2, particle_meta.colors.buffer().unwrap().slice(..));
            pass.draw(0..6, range.start as u32..range.end as u32);
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
        let particle_buffers = world.get_resource::<ParticleMeta>().unwrap();
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
