use bevy::math::{curves::CurveFixed, interpolation::Lerp};
use std::ops::Range;

#[inline]
pub fn from_constant<T: Lerp + Clone>(value: T) -> CurveFixed<Range<T>> {
    from_range(value.clone()..value.clone())
}

#[inline]
pub fn from_range<T: Lerp + Clone>(value: Range<T>) -> CurveFixed<Range<T>> {
    from_vec(vec![value])
}

#[inline]
pub fn from_constant_vec<T: Lerp + Clone>(keyframes: Vec<T>) -> CurveFixed<Range<T>> {
    from_vec(
        keyframes
            .into_iter()
            .map(|value| value.clone()..value.clone())
            .collect(),
    )
}

#[inline]
pub fn from_vec<T: Lerp>(keyframes: Vec<T>) -> CurveFixed<T> {
    CurveFixed::from_keyframes(keyframes.len() as f32, 0, keyframes)
}
