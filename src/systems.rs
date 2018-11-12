use amethyst::{
    assets::AssetStorage,
    core::{
        nalgebra::{UnitQuaternion, Vector2, Vector3},
        timing::Time,
        transform::Transform,
    },
    ecs::{
        prelude::{Entities, Entity, Join, LazyUpdate, Read, ReadStorage, System, WriteStorage},
        ReadExpect, WriteExpect,
    },
    audio::{
        output::Output,
        Source,
    },
    input::InputHandler,
    ui::UiText,
};
use crate::{
    resources::{AsteroidResource, BulletResource, Collider, GameResource, RandomGen, Score},
    BoundingVolume, Bullet, ConstrainedObject, Physical, Ship, ARENA_HEIGHT, ARENA_WIDTH,
    audio::Sounds,
};
use log::{error, trace};
use ncollide2d::broad_phase::{BroadPhase, DBVTBroadPhase};
use smallvec::SmallVec;

#[derive(Default)]
pub struct GlobalInputSystem {
    immortal_down: bool,
}

impl<'s> System<'s> for GlobalInputSystem {
    type SystemData = (
        Read<'s, InputHandler<String, String>>,
        WriteExpect<'s, GameResource>,
    );

    fn run(&mut self, (input, mut game): Self::SystemData) {
        let immortal_pressed = input.action_is_down("immortal").unwrap_or(false);

        if immortal_pressed {
            if !self.immortal_down {
                game.modifiers.player_is_immortal = !game.modifiers.player_is_immortal;
                self.immortal_down = true;
            }

            return;
        } else {
            if self.immortal_down {
                self.immortal_down = false;
            }
        }

        let restart_pressed = input.action_is_down("restart").unwrap_or(false);

        if restart_pressed {
            game.restart = true;
        }
    }
}

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
        ReadExpect<'s, Sounds>,
        Read<'s, AssetStorage<Source>>,
        Option<Read<'s, Output>>,
        Entities<'s>,
        Read<'s, LazyUpdate>,
    );

    fn run(&mut self, system: Self::SystemData) {
        let (
            mut ships,
            mut physicals,
            locals, time, input, bullet_resource, rand,
            sounds,
            audio_storage,
            audio,
            entities,
            lazy,
        ) = system;

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

        if !new_bullets.is_empty() {
            sounds.pew_sfx.play(&rand, &audio_storage, audio.as_ref().map(|o| &**o));
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

            let r = || (rand.next_f32() - 0.5) * 2.0 * self.max_velocity;
            let velocity = Vector2::new(r(), r());

            spawn_asteroid(
                &entities,
                &lazy,
                &rand,
                &asteroid_resource,
                local,
                scale,
                velocity,
                self.max_rotation,
                false,
            );

            self.time_to_spawn = rand.next_f32() * self.average_spawn_time;
        }
    }
}

fn spawn_asteroid(
    entities: &Entities,
    lazy: &Read<LazyUpdate>,
    rand: &ReadExpect<RandomGen>,
    asteroid_resource: &ReadExpect<AsteroidResource>,
    mut local: Transform,
    scale: f32,
    velocity: Vector2<f32>,
    max_rotation: f32,
    defer_adding_bounds: bool,
) {
    *local.scale_mut() = Vector3::new(scale, scale, 1.0f32);

    let mut physical = Physical::new();
    physical.velocity = velocity;
    physical.rotation = max_rotation * rand.next_f32();

    let e = entities.create();

    lazy.insert(e, local);
    lazy.insert(e, physical);
    lazy.insert(e, ConstrainedObject);
    lazy.insert(e, asteroid_resource.new_sprite_render(rand));

    let bounding_volume = if defer_adding_bounds {
        asteroid_resource.create_bounding_volume(e, scale, Collider::DeferredAsteroid)
    } else {
        asteroid_resource.create_bounding_volume(e, scale, Collider::Asteroid)
    };

    lazy.insert(e, bounding_volume);
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
        WriteStorage<'s, BoundingVolume>,
        ReadStorage<'s, Transform>,
        WriteExpect<'s, GameResource>,
        WriteStorage<'s, UiText>,
        WriteExpect<'s, Score>,
        Read<'s, LazyUpdate>,
        ReadExpect<'s, AsteroidResource>,
        ReadExpect<'s, RandomGen>,
        ReadExpect<'s, Sounds>,
        Read<'s, AssetStorage<Source>>,
        Option<Read<'s, Output>>,
        Entities<'s>,
    );

    fn run(&mut self, data: Self::SystemData) {
        use std::collections::HashSet;

        let (
            mut bounding_volumes,
            locals,
            mut game,
            mut text,
            mut score,
            lazy,
            asteroids_resource,
            rand,
            sounds,
            audio_storage,
            audio,
            entities,
        ) = data;

        let mut broad_phase = DBVTBroadPhase::new(0f32);

        let mut deferred = HashSet::new();
        let mut seen = HashSet::new();

        for (local, bounding_volume) in (&locals, &bounding_volumes).join() {
            let _ = bounding_volume.apply_to_broad_phase(local, &mut broad_phase);

            if let Collider::DeferredAsteroid(e) = bounding_volume.collider {
                deferred.insert(e);
            }
        }

        let mut spawned = 0;

        broad_phase.update(&mut |a, b| a != b, &mut |a, b, _| {
            use self::Collider::*;

            // play the appropriate sound.
            match (*a, *b) {
                (Asteroid(_), _) | (_, Asteroid(_)) => {
                    sounds.collision_sfx.play(&rand, &audio_storage, audio.as_ref().map(|o| &**o));
                }
                _ => {}
            }

            match (*a, *b) {
                (DeferredAsteroid(a), DeferredAsteroid(b)) => {
                    seen.insert(a);
                    seen.insert(b);
                    return;
                }
                (Bullet(_), Ship(_)) | (Ship(_), Bullet(_)) => {
                    return;
                }
                (Ship(_), _) | (_, Ship(_)) => {
                    if game.modifiers.player_is_immortal {
                        return;
                    }

                    game.modifiers.player_is_dead = true;
                }
                (Bullet(_), Asteroid(a)) | (Asteroid(a), Bullet(_)) => {
                    sounds.explosion_sfx.play(&rand, &audio_storage, audio.as_ref().map(|o| &**o));

                    score.asteroids += 1;

                    if let Some(text) = text.get_mut(score.score_text) {
                        text.text = score.asteroids.to_string();
                    }

                    // explode into smaller asteroids.
                    if let Some((local, volume)) = asteroid_data(a, &bounding_volumes, &locals) {
                        spawned += spawn_asteroid_cluster(
                            local,
                            volume,
                            &entities,
                            &lazy,
                            &asteroids_resource,
                            &rand,
                        );
                    }
                }
                // this one is _interesting_, cause now we get to break up asteroids if they are
                // big enough!
                (Asteroid(a), Asteroid(b)) => {
                    let a = asteroid_data(a, &bounding_volumes, &locals);
                    let b = asteroid_data(b, &bounding_volumes, &locals);

                    if let (Some(a), Some(b)) = (a, b) {
                        let mut local = Transform::default();
                        *local.translation_mut() = (a.0.translation() + b.0.translation()) / 2.0;
                        let volume = a.1 + b.1;

                        spawned += spawn_asteroid_cluster(
                            local,
                            volume,
                            &entities,
                            &lazy,
                            &asteroids_resource,
                            &rand,
                        );
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

        // undefer deferred
        for e in deferred {
            if !seen.contains(&e) {
                if let Some(volume) = bounding_volumes.get_mut(e) {
                    volume.collider = Collider::Asteroid(e);
                }
            }
        }

        if spawned > 0 {
            trace!("SPAWNED: {}", spawned);
        }

        fn asteroid_data(
            e: Entity,
            bounding_volumes: &WriteStorage<BoundingVolume>,
            locals: &ReadStorage<Transform>,
        ) -> Option<(Transform, f32)> {
            use std::f32::consts;

            let volume = match bounding_volumes.get(e) {
                Some(volume) => volume,
                None => return None,
            };

            let local = match locals.get(e) {
                Some(local) => local.clone(),
                None => return None,
            };

            Some((local.clone(), volume.shape.radius().powf(2.0) * consts::PI))
        }

        fn spawn_asteroid_cluster(
            local: Transform,
            c: f32,
            entities: &Entities,
            lazy: &Read<LazyUpdate>,
            asteroids_resource: &ReadExpect<AsteroidResource>,
            rand: &ReadExpect<RandomGen>,
        ) -> usize {
            use std::f32::consts;

            let min_area = AsteroidResource::MIN_RADIUS.powf(2.0) * consts::PI;

            let mut angle = 0.0f32;

            let mut count = 0;

            if c < (min_area * 3.0) {
                return count;
            }

            for _ in 0..2 {
                count += 1;
                angle += rand.next_f32() * consts::PI;

                let rotation = UnitQuaternion::from_axis_angle(&Vector3::z_axis(), angle);
                let velocity = rotation * Vector3::x() * 100.0 * rand.next_f32();
                let velocity = Vector2::new(velocity.x, velocity.y);

                spawn_asteroid(
                    entities,
                    lazy,
                    rand,
                    asteroids_resource,
                    local.clone(),
                    1.0,
                    velocity,
                    0.10,
                    true,
                );
            }

            return count;
        }
    }
}

pub struct HandleUiSystem;

impl<'s> System<'s> for HandleUiSystem {
    type SystemData = (
        ReadExpect<'s, GameResource>,
        WriteStorage<'s, UiText>,
        WriteExpect<'s, Score>,
    );

    fn run(&mut self, (game, mut text, mut score): Self::SystemData) {
        if game.modifiers != score.current_modifiers {
            score.current_modifiers = game.modifiers;

            if let Some(text) = text.get_mut(score.modifiers_text) {
                text.text = game.modifiers.as_text();
            }
        }
    }
}
