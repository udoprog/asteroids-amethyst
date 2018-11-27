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
    use amethyst::prelude::{dynamic::Application, Config};
    use crate::{
        audio::Silent,
        bundle::{GlobalBundle, MainBundle},
        states::State,
    };

    amethyst::start_logger(Default::default());

    let app = opts();
    let matches = app.get_matches();

    let app_root = application_root_dir();

    let display_config_path = format!("{}/resources/display.ron", app_root);
    let config = DisplayConfig::load(&display_config_path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawFlat2D::new().with_transparency(ColorMask::all(), ALPHA, None))
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

    let mut app = Application::build(assets_dir, State::Main)?
        .with_defaults()
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path)?,
        )?.with_bundle(RenderBundle::new(pipe, Some(config)).with_sprite_sheet_processor())?
        .with_bundle(TransformBundle::new())?
        .with_bundle(AudioBundle::new(|_: &mut Silent| None))?
        .with_bundle(UiBundle::<String, String>::new())?
        .with_bundle(GlobalBundle)?
        .with_bundle(MainBundle)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        ).with_state(
            State::Main,
            states::MainState {
                player_is_immortal: matches.is_present("god"),
            },
        )?.with_state(State::Paused, states::PausedState)?
        .build()?;

    app.run();
    Ok(())
}
