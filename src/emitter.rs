use bevy_core::Time;
use bevy_ecs::{component::Component, prelude::{Query, Res}};
use bevy_tasks::ComputeTaskPool;
use crate::{particles::Particles, curve::MinMaxCurve};

pub trait Emitter: Component {
    fn emit(&mut self, particles: &mut Particles);
}

pub struct EmitterTimer {

}

pub fn emit_particles<T: Emitter>(
    time: Res<Time>,
    compute_task_pool: Res<ComputeTaskPool>,
    mut particles: Query<(&mut T, &mut Particles, &mut EmitterTimer)>
) {
}
