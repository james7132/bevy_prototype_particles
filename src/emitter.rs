use crate::{
    curve::MinMaxCurve,
    particles::{ParticleParams, Particles},
};
use bevy::{ecs::component::Component, prelude::*, tasks::ComputeTaskPool};
use rand::Rng;
use std::{ops::Range, time::Duration};

#[derive(Debug, Clone)]
pub struct EmitterBurst {
    pub count: Range<usize>,
    pub wait: Duration,
}

pub trait EmitterModifier: Send + Sync + 'static {
    fn modify(&mut self, particle: &mut ParticleParams);
}

pub struct ParticleEmitter {
    next_burst: Timer,
    burst_idx: usize,
    bursts: Vec<EmitterBurst>,
    shape: EmitterShape,
    modifiers: Vec<Box<dyn EmitterModifier>>,
}

pub enum EmitterShape {
    Sphere { center: Vec3, radius: f32 },
    Hemisphere { center: Vec3, radius: f32 },
}

impl EmitterShape {
    fn sample(&self, rng: &mut impl Rng) -> ParticleParams {
        match self {
            Self::Sphere { radius, center } => Self::sample_sphere(*center, *radius, rng),
            Self::Hemisphere { radius, center } => Self::sample_hemisphere(*center, *radius, rng),
        }
    }

    fn sample_sphere(center: Vec3, radius: f32, rng: &mut impl Rng) -> ParticleParams {
        let position = sample_sphere(rng);
        let r = rng.gen_range(0.0..1.0);
        ParticleParams {
            position: position * r * radius + center,
            velocity: position,
            ..Default::default()
        }
    }

    fn sample_hemisphere(center: Vec3, radius: f32, rng: &mut impl Rng) -> ParticleParams {
        let mut position = sample_sphere(rng);
        position.y = f32::abs(position.y);
        let r = rng.gen_range(0.0..1.0);
        ParticleParams {
            position: position * r * radius + center,
            velocity: position,
            ..Default::default()
        }
    }
}

pub fn emit_particles(
    time: Res<Time>,
    compute_task_pool: Res<ComputeTaskPool>,
    mut particles: Query<(&mut ParticleEmitter, &mut Particles, &GlobalTransform)>,
) {
    particles.par_for_each_mut(
        &compute_task_pool,
        8,
        |(mut emitter, mut particles, transform)| {
            if !emitter.next_burst.finished() {
                return;
            }
            let local_to_world = transform.compute_matrix();
            let mut rng = rand::thread_rng();
            let EmitterBurst { count, wait } = emitter.bursts[emitter.burst_idx].clone();
            let count = rng.gen_range(count);

            let target_capacity = particles.len() + count;
            particles.reserve(target_capacity);
            for _ in 0..count {
                let mut params = emitter.shape.sample(&mut rng);
                params.position = local_to_world.transform_point3(params.position);
                params.velocity = local_to_world.transform_vector3(params.velocity);
                for modifier in emitter.modifiers.iter_mut() {
                    modifier.modify(&mut params);
                }
                particles.spawn(params);
            }

            emitter.next_burst.set_duration(wait);
            emitter.next_burst.reset();
            emitter.burst_idx = (emitter.burst_idx + 1) % emitter.bursts.len();
        },
    );
}

/// Select one point at random on the unit sphere.
fn sample_sphere(rng: &mut impl Rng) -> Vec3 {
    const TWO_PI: f32 = std::f32::consts::PI * 2.0;
    let theta = rng.gen_range(0.0..TWO_PI);
    let z = rng.gen_range(-1.0..1.0);
    let x = f32::sqrt(1.0 - z * z) * f32::cos(theta);
    let y = f32::sqrt(1.0 - z * z) * f32::sin(theta);

    Vec3::from((x, y, z))
}
