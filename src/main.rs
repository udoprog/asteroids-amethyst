use self::resources::Collider;
use amethyst::{
    audio::AudioBundle,
    core::{
        frame_limiter::FrameRateLimitStrategy,
        nalgebra::{self, Isometry2, Vector2},
        transform::{Transform, TransformBundle},
    },
    ecs::prelude::{Component, DenseVecStorage, Entity, NullStorage},
    input::InputBundle,
    prelude::*,
    renderer::{ColorMask, DisplayConfig, DrawSprite, Pipeline, RenderBundle, Stage, ALPHA},
    ui::{DrawUi, UiBundle},
    utils::application_root_dir,
};
use ncollide2d::{
    bounding_volume::{self, AABB},
    broad_phase::BroadPhase,
    shape::Ball,
};

mod audio;
mod asteroids;
mod bundle;
mod resources;
mod systems;
mod textures;

use std::time::Duration;

use clap::{App, Arg};

const ARENA_HEIGHT: f32 = 300.0;
const ARENA_WIDTH: f32 = 300.0;

fn opts() -> App<'static, 'static> {
    App::new("Asteroids!")
        .version("1.0")
        .author("John-John Tedro <udoprog@tedro.se>")
        .about("Asteroids! the Game")
        .arg(Arg::with_name("god")
             .long("god")
             .help("Want to be immortal? Now is your chance!"))
}

fn main() -> amethyst::Result<()> {
    use crate::{
        audio::Silent,
        asteroids::Asteroids,
    };

    amethyst::start_logger(Default::default());

    let app = opts();
    let matches = app.get_matches();

    let mut game = Asteroids::default();
    game.player_is_immortal = matches.is_present("god");

    let app_root = application_root_dir();

    let display_config_path = format!("{}/resources/display.ron", app_root);
    let config = DisplayConfig::load(&display_config_path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawSprite::new().with_transparency(ColorMask::all(), ALPHA, None))
            .with_pass(DrawUi::new()),
    );

    let key_bindings_path = {
        if cfg!(feature = "sdl_controller") {
            format!("{}/resources/input_controller.ron", app_root)
        } else {
            format!("{}/resources/input.ron", app_root)
        }
    };

    let assets_dir = format!("{}/assets/", app_root);

    let game_data = GameDataBuilder::default()
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path)?,
        )?.with_bundle(self::bundle::MainBundle)?
        .with_bundle(RenderBundle::new(pipe, Some(config)).with_sprite_sheet_processor())?
        .with_bundle(TransformBundle::new().with_dep(&["physics_system"]))?
        .with_bundle(AudioBundle::new(|_: &mut Silent| None))?
        .with_bundle(UiBundle::<String, String>::new())?;

    let mut game = Application::build(assets_dir, game)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        ).build(game_data)?;

    game.run();
    Ok(())
}

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
pub struct BoundingVolume {
    /// Bounding volume of the entity.
    pub shape: Ball<f32>,
    /// A registered collider.
    pub collider: Collider,
}

impl BoundingVolume {
    pub fn new(shape: Ball<f32>, collider: Collider) -> Self {
        Self { shape, collider }
    }

    pub fn from_local(entity: Entity, size: f32, collider: impl Fn(Entity) -> Collider) -> Self {
        let ball = Ball::new(size);
        let c = collider(entity);
        BoundingVolume::new(ball, c)
    }

    /// Apply this bounding volume to a broad phase.
    pub fn apply_to_broad_phase<B>(&self, local: &Transform, broad_phase: &mut B) -> AABB<f32>
    where
        B: BroadPhase<f32, AABB<f32>, Collider>,
    {
        let t = local.translation();
        let pos = Isometry2::new(Vector2::new(t.x, t.y), nalgebra::zero());
        let vol = bounding_volume::aabb(&self.shape, &pos);
        let _ = broad_phase.create_proxy(vol.clone(), self.collider);
        vol
    }
}

impl Component for BoundingVolume {
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
