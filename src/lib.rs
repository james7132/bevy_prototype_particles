use bevy::prelude::*;

mod curve;
mod emitter;
mod particles;
pub mod render;

pub use emitter::*;
pub use particles::*;

use bevy::render2::{
    core_pipeline, render_graph::RenderGraph, render_phase::DrawFunctions, RenderStage,
};
use render::{DrawParticle, ParticleMeta, ParticleNode, ParticleShaders};

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(particles::update_particles.system())
            .add_system(emitter::emit_particles.system());
        let render_app = app.sub_app_mut(0);
        render_app
            .add_system_to_stage(RenderStage::Extract, render::extract_particles.system())
            .add_system_to_stage(RenderStage::Prepare, render::prepare_particles.system())
            .add_system_to_stage(RenderStage::Queue, render::queue_particles.system())
            .init_resource::<ParticleShaders>()
            .init_resource::<ParticleMeta>();

        let draw_particle = DrawParticle::new(&mut render_app.world);
        render_app
            .world
            .get_resource::<DrawFunctions>()
            .unwrap()
            .write()
            .add(draw_particle);

        let render_world = app.sub_app_mut(0).world.cell();
        let mut graph = render_world.get_resource_mut::<RenderGraph>().unwrap();
        graph.add_node("particles", ParticleNode);
        graph
            .add_node_edge("particles", core_pipeline::node::MAIN_PASS_DEPENDENCIES)
            .unwrap();

        bevy::log::info!("TEST");
    }
}
