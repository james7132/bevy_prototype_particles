use bevy::{
    prelude::*,
    render::color::Color,
    render2::render_resource::{BufferUsage, BufferVec},
    tasks::ComputeTaskPool,
};

#[derive(Debug, Default, Clone)]
pub struct ParticleParams {
    pub position: Vec3,
    pub rotation: f32,
    pub size: f32,
    pub velocity: Vec3,
    pub angular_velocity: f32,
    pub color: Color,
    pub lifetime: f32,
}

#[derive(Debug, Clone)]
pub struct Particle<'a> {
    pub position: &'a Vec4,
    pub size: &'a f32,
    pub velocity: &'a Vec4,
    pub color: &'a Vec4,
    pub lifetime: &'a f32,
}

#[derive(Debug)]
pub struct ParticleMut<'a> {
    pub position: &'a mut Vec4,
    pub size: &'a mut f32,
    pub velocity: &'a mut Vec4,
    pub color: &'a mut Vec4,
    pub lifetime: &'a mut f32,
}

#[derive(Default, Clone)]
/// A container component for a batch of particles.
pub struct Particles {
    pub(crate) capacity: usize,
    // X, Y, Z - world coordinates
    // W - 1D rotation
    pub(crate) positions: Vec<Vec4>,
    pub(crate) sizes: Vec<f32>,
    pub(crate) colors: Vec<Vec4>,
    // X, Y, Z - world coordinates
    // W - 1D rotation
    pub(crate) velocities: Vec<Vec4>,
    pub(crate) lifetimes: Vec<f32>,
    pub(crate) start_lifetimes: Vec<f32>,
}

impl Particles {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            positions: Vec::with_capacity(capacity),
            sizes: Vec::with_capacity(capacity),
            colors: Vec::with_capacity(capacity),
            velocities: Vec::with_capacity(capacity),
            lifetimes: Vec::with_capacity(capacity),
            start_lifetimes: Vec::with_capacity(capacity),
        }
    }
    /// Gets a read-only reference to a particle.
    ///
    /// # Panics
    /// Panics if the provided index is out of bounds.
    pub fn get<'a>(&'a self, idx: usize) -> Particle<'a> {
        Particle {
            position: &self.positions[idx],
            size: &self.sizes[idx],
            velocity: &self.velocities[idx],
            color: &self.colors[idx],
            lifetime: &self.lifetimes[idx],
        }
    }

    /// Gets a mutable reference to a particle.
    ///
    /// # Panics
    /// Panics if the provided index is out of bounds.
    pub fn get_mut<'a>(&'a mut self, idx: usize) -> ParticleMut<'a> {
        ParticleMut {
            position: &mut self.positions[idx],
            size: &mut self.sizes[idx],
            velocity: &mut self.velocities[idx],
            color: &mut self.colors[idx],
            lifetime: &mut self.lifetimes[idx],
        }
    }

    /// Spawns a single particle with the given parameters.
    ///
    /// If spawning multiple at the same time, use `spawn_batch` instead.
    #[inline(always)]
    pub fn spawn(&mut self, params: ParticleParams) {
        self.positions
            .push(Vec4::from((params.position, params.rotation)));
        self.sizes.push(params.size);
        self.velocities
            .push(Vec4::from((params.velocity, params.angular_velocity)));
        self.colors.push(params.color.as_rgba_f32().into());
        self.lifetimes.push(0.0);
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
        self.sizes.extend(batch.sizes);
        self.velocities.extend(batch.velocities);
        self.lifetimes.extend(batch.lifetimes);
        self.colors.extend(batch.colors);
        self.start_lifetimes.extend(batch.start_lifetimes);
    }

    pub fn iter<'a>(&'a self) -> ParticleIter<'a> {
        ParticleIter {
            idx: 0,
            particles: self,
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> ParticleIterMut<'a> {
        ParticleIterMut {
            idx: 0,
            particles: self,
        }
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }

    pub fn capacity(&self) -> usize {
        self.positions.capacity()
    }

    pub fn reserve(&mut self, capacity: usize) {
        self.positions.reserve(capacity);
        self.sizes.reserve(capacity);
        self.velocities.reserve(capacity);
        self.lifetimes.reserve(capacity);
        self.colors.reserve(capacity);
        self.start_lifetimes.reserve(capacity);
    }

    pub fn clear(&mut self) {
        self.positions.clear();
        self.sizes.clear();
        self.velocities.clear();
        self.lifetimes.clear();
        self.colors.clear();
        self.start_lifetimes.clear();
    }

    /// Gets a ratio of how much of a particle's lifetime has passed. Will be 0.0 when the
    /// particle is newly spawned, and 1.0 or greater when the particle is about to be killed.
    pub fn lifetime_ratio(&self, idx: usize) -> f32 {
        let start_lifetime = self.start_lifetimes[idx];
        let lifetime = self.start_lifetimes[idx];
        (start_lifetime - lifetime) / start_lifetime
    }

    #[inline(always)]
    pub fn advance_particles(&mut self, delta_time: f32) {
        let mut active_count = self.len();
        let mut idx = 0;
        while idx < active_count {
            self.positions[idx] += self.velocities[idx] * delta_time;
            self.lifetimes[idx] += delta_time;
            if self.lifetimes[idx] >= self.start_lifetimes[idx] {
                self.kill(idx, active_count - 1);
                active_count -= 1;
            } else {
                idx += 1;
            }
        }
        if active_count < self.len() {
            self.flush(active_count);
        }
    }

    #[inline(always)]
    fn kill(&mut self, idx: usize, end: usize) {
        self.positions.swap(idx, end);
        self.sizes.swap(idx, end);
        self.velocities.swap(idx, end);
        self.lifetimes.swap(idx, end);
        self.colors.swap(idx, end);
        self.start_lifetimes.swap(idx, end);
    }

    #[inline(always)]
    fn flush(&mut self, len: usize) {
        self.positions.truncate(len);
        self.sizes.truncate(len);
        self.velocities.truncate(len);
        self.lifetimes.truncate(len);
        self.start_lifetimes.truncate(len);
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
                    size: &mut *particles.sizes.as_mut_ptr().add(self.idx),
                    velocity: &mut *particles.velocities.as_mut_ptr().add(self.idx),
                    color: &mut *particles.colors.as_mut_ptr().add(self.idx),
                    lifetime: &mut *particles.lifetimes.as_mut_ptr().add(self.idx),
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
    });
}
