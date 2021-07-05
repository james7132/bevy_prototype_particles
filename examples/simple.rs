use bevy::{prelude::*, render2::camera::PerspectiveCameraBundle, PipelinedDefaultPlugins};
use bevy_prototype_particles::*;
use rand::Rng;

fn create_scene(
    mut commands: Commands,
) {
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn create_particles(mut commands: Commands) {
    const PARTICLE_SYSTEM_COUNT: usize = 90;
    const PARTICLE_COUNT: usize = 10000;
    let mut rng = rand::thread_rng();
    for _ in 0..PARTICLE_SYSTEM_COUNT {
        let mut particles = Particles::new(PARTICLE_COUNT);
        for _ in 0..PARTICLE_COUNT {
            particles.spawn(ParticleParams {
                lifetime: rng.gen_range(1.0..100.0),
                ..Default::default()
            });
        }
        commands.spawn().insert(particles);
    }
}

fn debug(time: Res<Time>) {
    bevy::log::info!("{:?} ms", time.delta_seconds_f64() * 1000.0);
}

fn main() {
    App::new()
        .add_plugins(PipelinedDefaultPlugins)
        .add_plugin(ParticlePlugin)
        .add_startup_system(create_scene.system())
        .add_startup_system(create_particles.system())
        .add_system(debug.system())
        .run()
}
