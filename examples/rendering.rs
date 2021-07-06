use bevy::{prelude::*, render2::camera::PerspectiveCameraBundle, PipelinedDefaultPlugins};
use bevy_prototype_particles::*;
use rand::Rng;

fn create_scene(mut commands: Commands) {
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn create_particles(
    mut commands: Commands,
    mut materials: ResMut<Assets<ParticleMaterial>>,
    asset_server: Res<AssetServer>,
) {
    const PARTICLE_COUNT: usize = 100000;
    let mut rng = rand::thread_rng();
    let mut particles = Particles::new(PARTICLE_COUNT);
    for _ in 0..PARTICLE_COUNT {
        particles.spawn(ParticleParams {
            position: Vec3::from((
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0),
            )),
            rotation: 10.0,
            angular_velocity: rng.gen_range(-1.0..1.0),
            color: Color::rgba(
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..0.25),
            ),
            size: rng.gen_range(0.1..0.15),
            velocity: Vec3::from((
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0),
            )) * rng.gen_range(0.0..3.0),
            lifetime: rng.gen_range(1.0..1000.0),
            ..Default::default()
        });
    }
    commands
        .spawn()
        .insert(particles)
        .insert(materials.add(ParticleMaterial {
            base_color_texture: Some(asset_server.load("icon.png")),
        }));
}

fn main() {
    App::new()
        .add_plugins(PipelinedDefaultPlugins)
        .add_plugin(ParticlePlugin)
        .add_startup_system(create_scene.system())
        .add_startup_system(create_particles.system())
        .run()
}
