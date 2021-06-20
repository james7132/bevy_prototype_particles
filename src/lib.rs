use bevy_core::Time;
use bevy_ecs::prelude::{Query, Res};
use bevy_math::*;
use bevy_tasks::ComputeTaskPool;

#[derive(Debug, Clone)]
pub struct ParticleParams {
    pub position: Vec3,
    pub rotation: f32,
    pub size: f32,
    pub velocity: Vec3,
    pub angular_velocity: f32,
    pub lifetime: f32,
}

#[derive(Debug, Clone)]
pub struct Particle<'a> {
    pub position: &'a Vec3,
    pub rotation: &'a f32,
    pub size: &'a f32,
    pub velocity: &'a Vec3,
    pub angular_velocity: &'a f32,
    pub remaining_lifetime: &'a f32,
}

pub struct ParticleMut<'a> {
    pub position: &'a mut Vec3,
    pub rotation: &'a mut f32,
    pub size: &'a mut f32,
    pub velocity: &'a mut Vec3,
    pub angular_velocity: &'a mut f32,
    pub remaining_lifetime: &'a mut f32,
}

#[derive(Clone)]
/// A container component for a batch of particles.
pub struct Particles {
    positions: Vec<Vec3>,
    rotations: Vec<f32>,
    sizes: Vec<f32>,
    velocities: Vec<Vec3>,
    angular_velocities: Vec<f32>,
    remaining_lifetimes: Vec<f32>,
    start_lifetimes: Vec<f32>,
}

impl Particles {
    /// Gets a read-only reference to a particle.
    ///
    /// # Panics
    /// Panics if the provided index is out of bounds.
    pub fn get<'a>(&'a self, idx: usize) -> Particle<'a> {
        Particle {
            position: &self.positions[idx],
            rotation: &self.rotations[idx],
            size: &self.sizes[idx],
            velocity: &self.velocities[idx],
            angular_velocity: &self.angular_velocities[idx],
            remaining_lifetime: &self.remaining_lifetimes[idx],
        }
    }

    /// Gets a mutable reference to a particle.
    ///
    /// # Panics
    /// Panics if the provided index is out of bounds.
    pub fn get_mut<'a>(&'a mut self, idx: usize) -> ParticleMut<'a> {
        ParticleMut {
            position: &mut self.positions[idx],
            rotation: &mut self.rotations[idx],
            size: &mut self.sizes[idx],
            velocity: &mut self.velocities[idx],
            angular_velocity: &mut self.angular_velocities[idx],
            remaining_lifetime: &mut self.remaining_lifetimes[idx],
        }
    }

    /// Spawns a single particle with the given parameters.
    ///
    /// If spawning multiple at the same time, use `spawn_batch` instead.
    #[inline(always)]
    pub fn spawn(&mut self, params: ParticleParams) {
        self.positions.push(params.position);
        self.rotations.push(params.rotation);
        self.sizes.push(params.size);
        self.velocities.push(params.velocity);
        self.angular_velocities.push(params.angular_velocity);
        self.remaining_lifetimes.push(params.lifetime);
        self.start_lifetimes.push(params.lifetime);
    }

    /// Spawns a batch of particles with the given parameters.
    pub fn spawn_batch(&mut self, batch: impl Iterator<Item = ParticleParams>) {
        let (lower, upper) = batch.size_hint();
        self.reserve(self.len() + upper.unwrap_or(lower));
        for param in batch {
            self.spawn(param);
        }
    }

    /// Consumes another Particles instance and merges in it's particles.
    pub fn merge(&mut self, batch: impl Into<Particles>) {
        let batch = batch.into();
        self.positions.extend(batch.positions);
        self.rotations.extend(batch.rotations);
        self.sizes.extend(batch.sizes);
        self.velocities.extend(batch.velocities);
        self.angular_velocities.extend(batch.angular_velocities);
        self.remaining_lifetimes.extend(batch.remaining_lifetimes);
        self.start_lifetimes.extend(batch.start_lifetimes);
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }

    pub fn capacity(&self) -> usize {
        self.positions.capacity()
    }

    pub fn reserve(&mut self, capacity: usize) {
        self.positions.reserve(capacity);
        self.rotations.reserve(capacity);
        self.sizes.reserve(capacity);
        self.velocities.reserve(capacity);
        self.angular_velocities.reserve(capacity);
        self.remaining_lifetimes.reserve(capacity);
        self.start_lifetimes.reserve(capacity);
    }

    pub fn clear(&mut self) {
        self.positions.clear();
        self.rotations.clear();
        self.sizes.clear();
        self.velocities.clear();
        self.angular_velocities.clear();
        self.remaining_lifetimes.clear();
        self.start_lifetimes.clear();
    }

    #[inline(always)]
    fn advance_particles(&mut self, delta_time: f32) {
        for lifetime in self.remaining_lifetimes.iter_mut() {
            *lifetime -= delta_time;
        }
    }

    #[inline(always)]
    fn move_particles(&mut self, delta_time: f32) {
        for (position, velocity) in self.positions.iter_mut().zip(self.velocities.iter()) {
            *position += *velocity * delta_time;
        }
    }

    #[inline(always)]
    fn rotate_particles(&mut self, delta_time: f32) {
        for (rotation, angular_velocity) in self
            .rotations
            .iter_mut()
            .zip(self.angular_velocities.iter())
        {
            *rotation += *angular_velocity * delta_time;
        }
    }

    #[inline(always)]
    fn kill_particles(&mut self) {
        let mut active_count = self.len();
        let mut current_idx = 0;
        while current_idx < active_count {
            if self.remaining_lifetimes[current_idx] > 0.0 {
                self.kill(current_idx, active_count - 1);
                active_count -= 1;
            } else {
                current_idx += 1;
            }
        }
        debug_assert!(active_count <= self.len());
        if active_count < self.len() {
            self.flush(active_count);
        }
    }

    #[inline(always)]
    fn kill(&mut self, idx: usize, end: usize) {
        self.positions.swap(idx, end);
        self.rotations.swap(idx, end);
        self.sizes.swap(idx, end);
        self.velocities.swap(idx, end);
        self.angular_velocities.swap(idx, end);
        self.remaining_lifetimes.swap(idx, end);
        self.start_lifetimes.swap(idx, end);
    }

    #[inline(always)]
    fn flush(&mut self, len: usize) {
        self.positions.truncate(len);
        self.rotations.swap_remove(len);
        self.sizes.swap_remove(len);
        self.velocities.swap_remove(len);
        self.angular_velocities.swap_remove(len);
        self.remaining_lifetimes.swap_remove(len);
        self.start_lifetimes.swap_remove(len);
    }
}

/// An iterator of read-only particles.
pub struct ParticleIter<'a> {
    idx: usize,
    particles: &'a Particles,
}

impl<'a> Iterator for ParticleIter<'a> {
    type Item = Particle<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.particles.len() {
            None
        } else {
            let particle = self.particles.get(self.idx);
            self.idx += 1;
            Some(particle)
        }
    }
}

/// An iterator of mutable particles.
pub struct ParticleIterMut<'a> {
    idx: usize,
    particles: &'a mut Particles,
}

impl<'a> Iterator for ParticleIterMut<'a> {
    type Item = ParticleMut<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.particles.len() {
            None
        } else {
            unsafe {
                let particles = &mut self.particles;
                let particle = ParticleMut {
                    position: &mut *particles.positions.as_mut_ptr().add(self.idx),
                    rotation: &mut *particles.rotations.as_mut_ptr().add(self.idx),
                    size: &mut *particles.sizes.as_mut_ptr().add(self.idx),
                    velocity: &mut *particles.velocities.as_mut_ptr().add(self.idx),
                    angular_velocity: &mut *particles.angular_velocities.as_mut_ptr().add(self.idx),
                    remaining_lifetime: &mut *particles
                        .remaining_lifetimes
                        .as_mut_ptr()
                        .add(self.idx),
                };
                self.idx += 1;
                Some(particle)
            }
        }
    }
}

pub fn update_particles(
    time: Res<Time>,
    compute_task_pool: Res<ComputeTaskPool>,
    mut particles: Query<&mut Particles>,
) {
    let delta_time = time.delta_seconds_f64() as f32;
    particles.par_for_each_mut(&compute_task_pool, 8, |mut particles| {
        particles.advance_particles(delta_time);
        particles.kill_particles();
        particles.move_particles(delta_time);
        particles.rotate_particles(delta_time);
    });
}
