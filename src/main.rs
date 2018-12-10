use amethyst::{
    audio::AudioBundle,
    core::{frame_limiter::FrameRateLimitStrategy, transform::TransformBundle},
    input::InputBundle,
    renderer::{ColorMask, DisplayConfig, DrawFlat2D, Pipeline, RenderBundle, Stage, ALPHA},
    ui::{DrawUi, UiBundle},
    utils::application_root_dir,
};

mod audio;
mod bundle;
mod components;
mod resources;
mod states;
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
    use amethyst::{
        shred::DispatcherBuilder,
        core::bundle::SystemBundle,
        prelude::{Application, Config, GameDataBuilder}
    };
    use crate::{
        audio::Silent,
        states::{MainGameState, DataBuilder},
        bundle::{GlobalBundle, MainBundle},
    };

    amethyst::start_logger(Default::default());

    let app = opts();
    let matches = app.get_matches();

    let mut game = MainGameState::default();
    game.player_is_immortal = matches.is_present("god");

    let app_root = application_root_dir()?;

    let display_config_path = app_root.join("resources/display.ron");
    let config = DisplayConfig::load(&display_config_path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawFlat2D::new().with_transparency(ColorMask::all(), ALPHA, None))
            .with_pass(DrawUi::new()),
    );

    let key_bindings_path = {
        if cfg!(feature = "sdl_controller") {
            app_root.join("resources/input_controller.ron")
        } else {
            app_root.join("resources/input.ron")
        }
    };

    let assets_dir = app_root.join("assets");

    let base = GameDataBuilder::default()
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path)?,
        )?
        .with_bundle(RenderBundle::new(pipe, Some(config)).with_sprite_sheet_processor())?
        .with_bundle(TransformBundle::new())?
        .with_bundle(AudioBundle::new(|_: &mut Silent| None))?
        .with_bundle(UiBundle::<String, String>::new())?
        .with_bundle(GlobalBundle)?;

    let mut main = DispatcherBuilder::default();
    MainBundle.build(&mut main)?;

    let mut game = Application::build(assets_dir, game)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .build(DataBuilder { base, main })?;

    game.run();
    Ok(())
}
