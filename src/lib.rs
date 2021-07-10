use bevy::prelude::*;

pub mod curve;
mod emitter;
mod material;
pub mod modifiers;
mod particles;
mod render;

pub use emitter::*;
pub use material::*;
use modifiers::*;
pub use particles::*;
pub use render::*;

use bevy::{
    ecs::prelude::*,
    render2::{core_pipeline, render_graph::RenderGraph, render_phase::DrawFunctions, RenderStage},
};
use render::{DrawParticle, ParticleMeta, ParticleNode, ParticleShaders};

const PARTICLE_UPDATE: &str = "particle_update";

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ParticleMaterialPlugin)
            .add_system(particles::update_particles.system().label(PARTICLE_UPDATE))
            .add_system(emitter::emit_particles.system().after(PARTICLE_UPDATE))
            .register_particle_modifier::<ConstantForce>()
            .register_particle_modifier::<ColorByLifetime>()
            .register_particle_modifier::<SizeOverLifetime>();

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
    }
}

pub trait ParticleModifierAppExt {
    fn register_particle_modifier<T: ParticleModifier>(&mut self) -> &mut Self;
}

impl ParticleModifierAppExt for App {
    fn register_particle_modifier<T: ParticleModifier>(&mut self) -> &mut Self {
        self.add_system(
            modifiers::apply_particle_modifier::<T>
                .system()
                .before(PARTICLE_UPDATE),
        );
        self
    }
}
