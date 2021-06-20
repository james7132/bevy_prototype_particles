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
    x: MinMaxCurve,
    y: MinMaxCurve,
    z: MinMaxCurve,
}

#[derive(Debug, Clone)]
pub struct LifetimeByEmitterSpeed {
    curve: MinMaxCurve,
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
pub struct SizeBySpeed {}

#[derive(Debug, Clone)]
pub struct SizeOverLifetime {}

#[derive(Debug, Clone)]
pub struct VelocityOverLifetime {}
