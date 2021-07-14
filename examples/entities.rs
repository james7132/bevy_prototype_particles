use bevy::prelude::*;
use bevy::tasks::ComputeTaskPool;

use rand::Rng;

fn create_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

struct Particle {
    position: Vec4,
    velocity: Vec4,
    lifetime: f32,
}

fn create_particles(mut commands: Commands) {
    const PARTICLE_SYSTEM_COUNT: usize = 90;
    const PARTICLE_COUNT: usize = 10000;
    let mut rng = rand::thread_rng();
    for _ in 0..PARTICLE_SYSTEM_COUNT {
        for _ in 0..PARTICLE_COUNT {
            commands.spawn().insert(Particle {
                position: Vec4::ZERO,
                velocity: Vec4::ZERO,
                lifetime: rng.gen_range(100.0..1000.0),
            });
        }
    }
}

fn update_particles(
    time: Res<Time>,
    compute_task_pool: Res<ComputeTaskPool>,
    mut particles: Query<&mut Particle>,
) {
    let dt = time.delta_seconds_f64() as f32;
    particles.par_for_each_mut(&compute_task_pool, 32, move |mut particle| {
        let velocity = particle.velocity * dt;
        particle.position += velocity;
        particle.lifetime -= dt;
    });
}

fn debug(time: Res<Time>) {
    bevy::log::info!("{:?} ms", time.delta_seconds_f64() * 1000.0);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(create_scene.system())
        .add_startup_system(create_particles.system())
        .add_system(debug.system())
        .add_system(update_particles.system())
        .run()
}
