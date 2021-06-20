use bevy_ecs::system::IntoSystem;
use bevy_app::{AppBuilder, Plugin};

mod curve;
pub mod particles;
pub mod emitter;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, builder: &mut AppBuilder) {
        builder
            .add_system(particles::update_particles.system());
    }
}
