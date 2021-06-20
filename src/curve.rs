pub enum MinMaxCurve {
    Constant(f32),
    Curve(Curve),
    RangeCurve {
        min: Curve,
        max: Curve,
    }
}

impl MinMaxCurve {
    pub fn evaluate(&self, time: f32, lerp_factor: f32) -> f32 {
        match self {
            Self::Constant(value) => *value,
            Self::Curve(curve) => curve.evaluate(time),
            Self::RangeCurve { min, max } => {
                // TODO(james7132): Use the Lerp trait when available
                let min = min.evaluate(time);
                let max = max.evaluate(time);
                (lerp_factor * (max - min)) + min
            }
        }
    }
}

pub struct Curve {
}

impl Curve {
    pub fn evaluate(&self, time: f32) -> f32 {
        0.0
    }
}
