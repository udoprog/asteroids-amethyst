use amethyst::{
    audio::AudioBundle,
    core::{frame_limiter::FrameRateLimitStrategy, transform::TransformBundle},
    input::InputBundle,
    renderer::{ColorMask, DisplayConfig, DrawSprite, Pipeline, RenderBundle, Stage, ALPHA},
    ui::{DrawUi, UiBundle},
    utils::application_root_dir,
};

mod asteroids;
mod audio;
mod bundle;
mod components;
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
        .arg(
            Arg::with_name("god")
                .long("god")
                .help("Want to be immortal? Now is your chance!"),
        )
}

fn main() -> amethyst::Result<()> {
    use amethyst::prelude::{Application, Config, GameDataBuilder};
    use crate::{asteroids::Asteroids, audio::Silent};

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
