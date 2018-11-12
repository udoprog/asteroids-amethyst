use amethyst::{
    core::{
        nalgebra::{Vector2, Vector3},
        timing::Time,
        transform::Transform,
    },
    ecs::{
        prelude::{Entities, Join, LazyUpdate, Read, ReadStorage, System, WriteStorage},
        ReadExpect, WriteExpect,
    },
    input::InputHandler,
    ui::UiText,
};
use crate::{
    resources::{AsteroidResource, BulletResource, Collider, GameResource, RandomGen, Score},
    BoundingVolume, Bullet, ConstrainedObject, Physical, Ship, ARENA_HEIGHT, ARENA_WIDTH,
};
use log::error;
use ncollide2d::broad_phase::{BroadPhase, DBVTBroadPhase};
use smallvec::SmallVec;

pub struct ShipInputSystem;

/// Handle inputs and mutate world accordingly.
///
/// * Applies rotation (axes `rotate`) and acceleration (axes `accelerate`) to your ship.
/// * Spawns bullets on `shoot` action..
impl<'s> System<'s> for ShipInputSystem {
    type SystemData = (
        WriteStorage<'s, Ship>,
        WriteStorage<'s, Physical>,
        ReadStorage<'s, Transform>,
        Read<'s, Time>,
        Read<'s, InputHandler<String, String>>,
        ReadExpect<'s, BulletResource>,
        ReadExpect<'s, RandomGen>,
        Entities<'s>,
        Read<'s, LazyUpdate>,
    );

    fn run(&mut self, system: Self::SystemData) {
        let (mut ships, mut physicals, locals, time, input, bullet_resource, rand, entities, lazy) =
            system;

        let time_delta = time.delta_seconds();

        let rotate = input.axis_value("rotate");
        let accelerate = input.axis_value("accelerate");
        let shoot = input.action_is_down("shoot").unwrap_or(false);

        let mut new_bullets = SmallVec::<[NewBullet; 4]>::new();

        for (ship, physical, local) in (&mut ships, &mut physicals, &locals).join() {
            // handle acceleration.
            if let Some(acceleration) = accelerate {
                // velocity to add.
                let added = Vector3::y() * ship.acceleration * time_delta * acceleration as f32;

                // add the velocity in the direction of the ship.
                let added = local.rotation() * added;

                physical.velocity = physical.velocity + Vector2::new(added.x, added.y);

                // limit velocity by some maximum.
                let magnitude = physical.velocity.magnitude();

                if magnitude != 0f32 {
                    let factor = magnitude / physical.max_velocity;

                    if factor > 1.0f32 {
                        physical.velocity = physical.velocity / factor;
                    }
                }
            }

            // handle rotation
            if let Some(rotation) = rotate {
                physical.rotation = ship.rotation * time_delta * rotation as f32;
            } else {
                physical.rotation = 0f32;
            }

            // handle shooting with a reload.
            if ship.reload_timer <= 0.0f32 {
                if shoot {
                    ship.reload_timer = ship.time_to_reload;

                    let mut local = local.clone();

                    // apply a bit of jitter on the bullet positions.
                    let jitter = Vector3::x() * (rand.next_f32() - 0.5) * ship.bullet_jitter;
                    let jitter = local.rotation() * jitter;
                    *local.translation_mut() += jitter;

                    new_bullets.push(NewBullet {
                        local,
                        velocity: ship.bullet_velocity,
                    });
                }
            } else {
                ship.reload_timer -= time_delta;

                if ship.reload_timer < 0.0f32 {
                    ship.reload_timer = 0.0f32;
                }
            }
        }

        for new_bullet in new_bullets {
            let NewBullet { local, velocity } = new_bullet;

            let velocity = local.rotation() * Vector3::y() * velocity;

            let mut physical = Physical::new();
            physical.velocity = Vector2::new(velocity.x, velocity.y);

            let e = entities.create();

            lazy.insert(e, local);
            lazy.insert(e, physical);
            lazy.insert(e, ConstrainedObject);
            lazy.insert(e, bullet_resource.new_sprite_render());
            lazy.insert(e, Bullet::new());
            lazy.insert(e, bullet_resource.create_bounding_volume(e));
        }

        struct NewBullet {
            local: Transform,
            velocity: f32,
        }
    }
}

/// Limit objects within arena.
///
/// If an object goes out of bounds, moves it to the other side of the arena.
pub struct LimitObjectsSystem;

impl<'s> System<'s> for LimitObjectsSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, ConstrainedObject>,
    );

    fn run(&mut self, (mut locals, constrained): Self::SystemData) {
        for (local, _) in (&mut locals, &constrained).join() {
            let mut t = *local.translation();

            if t.x < 0f32 {
                t.x += ARENA_WIDTH;
            } else if t.x > ARENA_WIDTH {
                t.x -= ARENA_WIDTH;
            }

            if t.y < 0f32 {
                t.y += ARENA_HEIGHT;
            } else if t.y > ARENA_HEIGHT {
                t.y -= ARENA_HEIGHT;
            }

            *local.translation_mut() = t;
        }
    }
}

pub struct KillBulletsSystem;

impl<'s> System<'s> for KillBulletsSystem {
    type SystemData = (Entities<'s>, WriteStorage<'s, Bullet>, Read<'s, Time>);

    fn run(&mut self, system: Self::SystemData) {
        let (entities, mut bullets, time) = system;

        let time_delta = time.delta_seconds();

        for (e, bullet) in (&*entities, &mut bullets).join() {
            bullet.time_to_live -= time_delta;

            if bullet.time_to_live <= 0.0f32 {
                if let Err(e) = entities.delete(e) {
                    error!("failed to destroy entity: {}", e);
                }

                continue;
            }
        }
    }
}

/// System to spawn random asteroids.
///
/// Asteroids are always spawned by the lower and upper edges, but with random velocity vectors
/// capped by the parameters in this system.
pub struct RandomAsteroidSystem {
    pub time_to_spawn: f32,
    pub max_velocity: f32,
    pub max_rotation: f32,
    pub average_spawn_time: f32,
}

impl RandomAsteroidSystem {
    pub fn new() -> Self {
        Self {
            time_to_spawn: 2f32,
            max_velocity: 100f32,
            max_rotation: 15f32,
            average_spawn_time: 0.5f32,
        }
    }
}

impl<'s> System<'s> for RandomAsteroidSystem {
    type SystemData = (
        Entities<'s>,
        ReadExpect<'s, AsteroidResource>,
        ReadExpect<'s, RandomGen>,
        Read<'s, Time>,
        Read<'s, LazyUpdate>,
    );

    fn run(&mut self, system: Self::SystemData) {
        let (entities, asteroid_resource, rand, time, lazy) = system;

        self.time_to_spawn -= time.delta_seconds();

        if self.time_to_spawn <= 0.0f32 {
            let mut local = Transform::default();
            local.translation_mut().x = rand.next_f32() * ARENA_HEIGHT;
            local.translation_mut().y = ARENA_WIDTH;

            let scale = 1.0f32 + rand.next_f32();
            *local.scale_mut() = Vector3::new(scale, scale, 1.0f32);

            let mut physical = Physical::new();
            let x_vel = (rand.next_f32() - 0.5) * 2.0 * self.max_velocity;
            let y_vel = (rand.next_f32() - 0.5) * 2.0 * self.max_velocity;
            physical.velocity = Vector2::new(x_vel, y_vel);
            physical.rotation = self.max_rotation * rand.next_f32();

            let e = entities.create();

            lazy.insert(e, local);
            lazy.insert(e, physical);
            lazy.insert(e, ConstrainedObject);
            lazy.insert(e, asteroid_resource.new_sprite_render(&rand));
            lazy.insert(e, asteroid_resource.create_bounding_volume(e, scale));

            self.time_to_spawn = rand.next_f32() * self.average_spawn_time;
        }
    }
}

/// Applies physics to `Physical` entities.
///
/// The system applies velocity and rotation to the objects in the system.
pub struct PhysicsSystem;

impl<'s> System<'s> for PhysicsSystem {
    type SystemData = (
        ReadStorage<'s, Physical>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
    );

    fn run(&mut self, (physicals, mut locals, time): Self::SystemData) {
        let time_delta = time.delta_seconds();

        for (physical, local) in (&physicals, &mut locals).join() {
            // Apply existing velocity and rotational velocity.
            let movement = physical.velocity * time_delta;

            local.move_global(Vector3::new(movement.x, movement.y, 0f32));
            local.roll_local(physical.rotation * time_delta);
        }
    }
}

/// Handle very simple collisions through ncollide2d's broad-phase DBVT implementation.
///
/// It _should_ be good enough since we are using very simple primitive (and zero margins) to
/// detect collisions.
///
/// I'm a bit concerned about re-creating the phase for every frame, but we don't have a ton of
/// objects so it should be fine.
pub struct CollisionSystem;

impl<'s> System<'s> for CollisionSystem {
    type SystemData = (
        ReadStorage<'s, BoundingVolume>,
        ReadStorage<'s, Transform>,
        WriteExpect<'s, GameResource>,
        WriteStorage<'s, UiText>,
        WriteExpect<'s, Score>,
        Entities<'s>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (bounding_volumes, locals, mut game, mut text, mut score, entities) = data;

        let mut broad_phase = DBVTBroadPhase::new(0f32);

        let mut tests = Vec::new();

        for (local, bounding_volume) in (&locals, &bounding_volumes).join() {
            let vol = bounding_volume.apply_to_broad_phase(local, &mut broad_phase);
            tests.push((vol, bounding_volume.collider));
        }

        broad_phase.update(&mut |a, b| a != b, &mut |a, b, _| {
            use self::Collider::*;

            match (*a, *b) {
                (Bullet(_), Ship(_)) | (Ship(_), Bullet(_)) => {
                    return;
                }
                (Ship(_), _) | (_, Ship(_)) => {
                    game.player_is_dead = true;
                }
                (Bullet(_), Asteroid(_)) | (Asteroid(_), Bullet(_)) => {
                    score.asteroids += 1;

                    if let Some(text) = text.get_mut(score.score_text) {
                        text.text = score.asteroids.to_string();
                    }
                }
                _ => {}
            }

            if let Err(e) = entities.delete(a.entity()) {
                error!("failed to delete entity: {:?}: {}", a, e);
            }

            if let Err(e) = entities.delete(b.entity()) {
                error!("failed to delete entity: {:?}: {}", b, e);
            }
        });
    }
}
