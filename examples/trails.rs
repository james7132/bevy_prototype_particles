use bevy::{prelude::*, render2::camera::PerspectiveCameraBundle, PipelinedDefaultPlugins};
use bevy_prototype_particles::*;
use std::time::Duration;

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
    let particles = Particles::new(1000);
    let source = commands
        .spawn()
        .insert(particles.clone())
        .insert(materials.add(ParticleMaterial {
            base_color_texture: Some(asset_server.load("icon.png")),
        }))
        .insert(modifiers::ColorByLifetime {
            color: curve::from_vec(vec![
                Vec4::from((1.0, 1.0, 1.0, 0.5)),
                Vec4::from((1.0, 1.0, 0.0, 0.45)),
                Vec4::from((1.0, 0.0, 0.0, 0.4)),
                Vec4::from((0.2, 0.0, 0.0, 0.15)),
                Vec4::from((0.0, 0.0, 0.0, 0.0)),
            ]),
        })
        .insert(modifiers::SizeOverLifetime {
            size: curve::from_constant_vec(vec![0.3, 0.2, 0.0]),
        })
        .insert(modifiers::ConstantForce {
            acceleration_per_second: Vec3::from((0.0, 5.0, 0.0)),
        })
        .insert(Transform {
            translation: Vec3::from((0.0, -1.0, 0.0)),
            ..Default::default()
        })
        .insert(GlobalTransform::default())
        .insert(
            ParticleEmitter::hemisphere(Vec3::ZERO, 1.0)
                .add_burst(EmitterBurst {
                    count: 5..10,
                    wait: Duration::from_millis(300),
                })
                .with_default_color(Color::rgba(0.5, 0.0, 0.0, 0.1))
                .with_default_lifetime(2.0)
                .with_default_speed(0.3)
                .with_default_size(0.2)
                .build(),
        )
        .id();

    commands
        .spawn()
        .insert(particles)
        .insert(materials.add(ParticleMaterial {
            base_color_texture: Some(asset_server.load("icon.png")),
        }))
        .insert(modifiers::SizeOverLifetime {
            size: curve::from_constant_vec(vec![0.3, 0.2, 0.0]),
        })
        .insert(Transform {
            translation: Vec3::from((0.0, -1.0, 0.0)),
            ..Default::default()
        })
        .insert(GlobalTransform::default())
        .insert(TrailEmitter {
            tracking: source,
            lifetime: 1.0,
        });
}

fn main() {
    App::new()
        .add_plugins(PipelinedDefaultPlugins)
        .add_plugin(ParticlePlugin)
        .add_startup_system(create_scene.system())
        .add_startup_system(create_particles.system())
        .run()
}
