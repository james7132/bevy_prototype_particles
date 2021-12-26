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

use render::ParticleRenderPlugin;

const PARTICLE_UPDATE: &str = "particle_update";

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ParticleMaterialPlugin)
            .add_plugin(ParticleRenderPlugin)
            .add_system(particles::update_particles.label(PARTICLE_UPDATE))
            .add_system(emitter::emit_particles.after(PARTICLE_UPDATE))
            .add_system(emitter::trail_particles.after(PARTICLE_UPDATE))
            .register_particle_modifier::<ConstantForce>()
            .register_particle_modifier::<ColorByLifetime>()
            .register_particle_modifier::<SizeOverLifetime>();
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
