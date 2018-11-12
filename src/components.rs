use amethyst::{
    core::{
        nalgebra::{self, Isometry2, Vector2},
        transform::Transform,
    },
    ecs::prelude::{Component, DenseVecStorage, Entity, NullStorage},
};
use ncollide2d::{
    bounding_volume::{self, AABB},
    broad_phase::BroadPhase,
    shape::Ball,
};

#[derive(Debug)]
pub struct Physical {
    /// Current velocity and direction of the entity (units / s).
    pub velocity: Vector2<f32>,
    /// Maximum velocity (units / s).
    pub max_velocity: f32,
    /// Current rotation (radians / s).
    pub rotation: f32,
}

impl Physical {
    pub fn new() -> Self {
        Self {
            velocity: Vector2::new(0f32, 0f32),
            max_velocity: 100f32,
            rotation: 0f32,
        }
    }
}

impl Component for Physical {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct Bounded {
    /// Bounding volume of the entity.
    pub shape: Ball<f32>,
}

impl Bounded {
    pub fn new(shape: Ball<f32>) -> Self {
        Self { shape }
    }

    pub fn from_local(size: f32) -> Self {
        Bounded::new(Ball::new(size))
    }

    /// Apply this bounding volume to a broad phase.
    pub fn apply_to_broad_phase<B>(
        &self,
        collider: Collider,
        e: Entity,
        local: &Transform,
        broad_phase: &mut B,
    ) -> AABB<f32>
    where
        B: BroadPhase<f32, AABB<f32>, (Collider, Entity)>,
    {
        let t = local.translation();
        let pos = Isometry2::new(Vector2::new(t.x, t.y), nalgebra::zero());
        let vol = bounding_volume::aabb(&self.shape, &pos);
        let _ = broad_phase.create_proxy(vol.clone(), (collider, e));
        vol
    }
}

impl Component for Bounded {
    type Storage = DenseVecStorage<Self>;
}

pub struct Ship {
    /// Acceleration this ship experiences on input (units / s**2).
    pub acceleration: f32,
    /// Rotation ship experiences on input.
    pub rotation: f32,
    /// How long until reloaded.
    pub reload_timer: f32,
    /// How long it takes to reload.
    pub time_to_reload: f32,
    /// Bullet velocity.
    pub bullet_velocity: f32,
    /// Amount of jitter from original shooting position.
    pub bullet_jitter: f32,
}

impl Default for Ship {
    fn default() -> Ship {
        Ship {
            acceleration: 80f32,
            rotation: 180f32,
            reload_timer: 0f32,
            time_to_reload: 0.1f32,
            bullet_velocity: 150f32,
            bullet_jitter: 2.0f32,
        }
    }
}

impl Component for Ship {
    type Storage = DenseVecStorage<Self>;
}

pub struct Bullet {
    /// How many seconds this bullet should live.
    pub time_to_live: f32,
}

impl Bullet {
    pub fn new() -> Bullet {
        Bullet { time_to_live: 2f32 }
    }
}

impl Component for Bullet {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, Default)]
pub struct ConstrainedObject;

impl Component for ConstrainedObject {
    type Storage = NullStorage<Self>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Collider {
    Bullet,
    Ship,
    Asteroid,
    /// Asteroid can collide, but will not register collissions until it's gone one frame without
    /// collisions.
    DeferredAsteroid,
}

impl Component for Collider {
    type Storage = DenseVecStorage<Self>;
}
