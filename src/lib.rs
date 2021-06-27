use bevy::prelude::*;

mod curve;
mod emitter;
mod particles;

pub use emitter::*;
pub use particles::*;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(particles::update_particles.system())
            .add_system(emitter::emit_particles.system());
    }
}
