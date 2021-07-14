use crate::Particles;
use bevy::{
    core::Time,
    ecs::prelude::*,
    math::{
        curves::{Curve, CurveFixed},
        interpolation::Lerp,
        *,
    },
    tasks::ComputeTaskPool,
};
use std::ops::Range;

pub trait ParticleModifier: Send + Sync + 'static {
    fn apply(&self, particles: &mut Particles, delta_time: f32);
}

#[derive(Debug, Clone)]
pub struct ColorBySpeed {
    pub color: CurveFixed<Vec4>,
    pub range: Range<f32>,
}

#[derive(Debug, Clone)]
pub struct ColorByLifetime {
    pub color: CurveFixed<Vec4>,
}

impl ParticleModifier for ColorByLifetime {
    fn apply(&self, particles: &mut Particles, _: f32) {
        for idx in 0..particles.len() {
            // SAFE: idx is always a valid particle index.
            unsafe {
                let lifetime = particles.lifetime_ratio(idx);
                *particles.colors.get_unchecked_mut(idx) = self.color.sample(lifetime);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ForceOverLifetime {
    pub force: CurveFixed<Range<Vec3>>,
}

#[derive(Debug, Clone)]
pub struct ConstantForce {
    pub acceleration_per_second: Vec3,
}

impl ParticleModifier for ConstantForce {
    fn apply(&self, particles: &mut Particles, delta_time: f32) {
        let delta_velocity = Vec4::from((self.acceleration_per_second, 0.0)) * delta_time;
        for velocity in particles.velocities.iter_mut() {
            *velocity += delta_velocity;
        }
    }
}

#[derive(Debug, Clone)]
pub struct LifetimeByEmitterSpeed {
    pub lifetime: CurveFixed<Range<f32>>,
    pub range: Range<f32>,
}

#[derive(Debug, Clone)]
pub struct LimitVelocityOverLifetime {}

#[derive(Debug, Clone)]
pub struct Noise {}

#[derive(Debug, Clone)]
pub struct RotationBySpeed {
    pub curve: CurveFixed<Range<f32>>,
    pub range: Range<f32>,
}

#[derive(Debug, Clone)]
pub struct RotationOverLifetime {
    pub rotation: CurveFixed<Range<f32>>,
}

#[derive(Debug, Clone)]
pub struct SizeBySpeed {
    pub size: CurveFixed<Range<f32>>,
    pub range: Range<f32>,
}

#[derive(Debug, Clone)]
pub struct SizeOverLifetime {
    pub size: CurveFixed<Range<f32>>,
}

impl ParticleModifier for SizeOverLifetime {
    fn apply(&self, particles: &mut Particles, _delta_time: f32) {
        for idx in 0..particles.len() {
            // SAFE: idx is always a valid particle index.
            unsafe {
                let lifetime = particles.lifetime_ratio(idx);
                let range = self.size.sample(lifetime);
                let lerp_factor = particles.lerp_factors.get_unchecked(idx);
                *particles.sizes.get_unchecked_mut(idx) =
                    f32::lerp_unclamped(&range.start, &range.end, *lerp_factor);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct VelocityOverLifetime {}

pub fn apply_particle_modifier<T: ParticleModifier>(
    compute_task_pool: Res<ComputeTaskPool>,
    time: Res<Time>,
    mut particles: Query<(&T, &mut Particles)>,
) {
    let delta_time = time.delta_seconds_f64() as f32;
    particles.par_for_each_mut(&compute_task_pool, 8, |(modifier, mut particles)| {
        modifier.apply(&mut particles, delta_time);
    });
}
