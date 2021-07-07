use crate::curve::MinMaxCurve;
use crate::Particles;
use bevy::{core::Time, ecs::prelude::*, math::*, tasks::ComputeTaskPool};
use std::ops::Range;

pub trait ParticleModifier: Send + Sync + 'static {
    fn apply(&self, particles: &mut Particles, delta_time: f32);
}

#[derive(Debug, Clone)]
pub struct ColorBySpeed {
    // color: Curve<Color>,
    pub range: Range<f32>,
}

#[derive(Debug, Clone)]
pub struct ColorByLifetime {
    // color: Curve<Color>,
}

#[derive(Debug, Clone)]
pub struct ForceOverLifetime {
    // space: SimulationSpace,
    x: MinMaxCurve,
    y: MinMaxCurve,
    z: MinMaxCurve,
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
    curve: MinMaxCurve,
    pub range: Range<f32>,
}

#[derive(Debug, Clone)]
pub struct LimitVelocityOverLifetime {}

#[derive(Debug, Clone)]
pub struct Noise {}

#[derive(Debug, Clone)]
pub enum RotationBySpeed {
    ZOnly {
        curve: MinMaxCurve,
        range: Range<f32>,
    },
    AllAxes {
        x: MinMaxCurve,
        y: MinMaxCurve,
        z: MinMaxCurve,
        range: Range<f32>,
    },
}

#[derive(Debug, Clone)]
pub enum RotationOverLifetime {
    ZOnly {
        curve: MinMaxCurve,
        range: Range<f32>,
    },
    AllAxes {
        x: MinMaxCurve,
        y: MinMaxCurve,
        z: MinMaxCurve,
        range: Range<f32>,
    },
}

#[derive(Debug, Clone)]
pub struct SizeBySpeed {}

#[derive(Debug, Clone)]
pub struct SizeOverLifetime {}

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
