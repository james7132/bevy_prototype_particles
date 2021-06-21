use bevy::prelude::*;

mod curve;
pub mod emitter;
pub mod particles;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, builder: &mut AppBuilder) {
        builder
            .add_system(particles::update_particles.system())
            .add_system(emitter::emit_particles.system());
    }
}
