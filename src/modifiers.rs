use crate::curve::MinMaxCurve;

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
    pub x: MinMaxCurve,
    pub y: MinMaxCurve,
    pub z: MinMaxCurve,
}

#[derive(Debug, Clone)]
pub struct LifetimeByEmitterSpeed {
    pub curve: MinMaxCurve,
    pub range: Range<f32>,
}

#[derive(Debug, Clone)]
pub struct LimitVelocityOverLifetime {
}

#[derive(Debug, Clone)]
pub struct Noise {}

#[derive(Debug, Clone)]
pub enum RotationBySpeed {
    ZOnly {
        curve: MinMaxCurve,
        range: Range<f32>,
    }
    AllAxes {
        x: MinMaxCurve,
        y: MinMaxCurve,
        z: MinMaxCurve,
        range: Range<f32>,
    }
}

#[derive(Debug, Clone)]
pub enum RotationOverLifetime {
    ZOnly {
        curve: MinMaxCurve,
        range: Range<f32>,
    }
    AllAxes {
        x: MinMaxCurve,
        y: MinMaxCurve,
        z: MinMaxCurve,
        range: Range<f32>,
    }
}

#[derive(Debug, Clone)]
pub struct SizeBySpeed {
    pub curve: MinMaxCurve,
    pub range: Range<f32>,
}

#[derive(Debug, Clone)]
pub struct SizeOverLifetime {
    pub curve: MinMaxCurve,
}

#[derive(Debug, Clone)]
pub struct VelocityOverLifetime {}

pub fn size_over_lifetime(
    compute_task_pool: Res<ComputeTaskPool>,
    particles: Query<(&SizeOverLifetime, &mut Particles)>,
) {
    particles.par_for_each(&compute_task_pool, 8, |(module, mut particles)| {
        for idx in 0..particles.len() {
            let ratio = particles.lifetime_ratio(idx);
            particles.speeds[idx] = module.curve.evaluate(ratio, 1.0);
        }
    });
}

