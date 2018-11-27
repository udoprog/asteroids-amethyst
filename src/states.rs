use amethyst::{
    assets::Loader,
    core::transform::Transform,
    ecs::prelude::World,
    prelude::{
        dynamic::{StateCallback, Trans},
        Builder,
    },
    renderer::{Camera, Projection},
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};

use crate::{
    audio::initialise_audio,
    components::{Collider, ConstrainedObject, Physical, Ship},
    resources::{Asteroids, Bullets, Game, RandomGen, Score, Ships},
    ARENA_HEIGHT, ARENA_WIDTH,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    Main,
    Paused,
}

impl Default for State {
    fn default() -> Self {
        State::Main
    }
}

pub struct MainState {
    pub player_is_immortal: bool,
}

impl<E> StateCallback<State, E> for MainState {
    fn on_start(&mut self, world: &mut World) {
        Ships::initialize(world);
        Bullets::initialize(world);
        Asteroids::initialize(world);
        world.add_resource(RandomGen);

        let mut game = Game::default();
        game.modifiers.player_is_immortal = self.player_is_immortal;

        initialize_score(world, &game);

        world.add_resource(game);

        // Setup our game.
        initialise_ship(world);
        initialise_camera(world);
        initialise_audio(world);
    }

    fn update(&mut self, world: &mut World) -> Trans<State> {
        let Game { restart, .. } = *world.read_resource::<Game>();

        if restart {
            world.delete_all();
            return Trans::Switch(State::Main);
        }

        let Game { ref mut pause, .. } = *world.write_resource::<Game>();

        if *pause {
            // NB: prevent a pause cycle by resetting the pause when acted on.
            *pause = false;
            return Trans::Push(State::Paused);
        }

        Trans::None
    }
}

/// Initialise the camera.
fn initialise_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.translation_mut().z = 1.0;
    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0,
            ARENA_WIDTH,
            0.0,
            ARENA_HEIGHT,
        ))).with(transform)
        .build();
}

/// Initialises one ship in the middle-ish of the arena.
fn initialise_ship(world: &mut World) {
    use crate::{ARENA_HEIGHT, ARENA_WIDTH};

    let mut local = Transform::default();
    local.set_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0, 0.0);

    let sprite_render = {
        let ship_resource = world.read_resource::<Ships>();
        ship_resource.new_sprite_render()
    };

    let bounding_volume = {
        let ship_resource = world.read_resource::<Ships>();
        ship_resource.new_bounded()
    };

    world
        .create_entity()
        .with(sprite_render)
        .with(Ship::default())
        .with(Physical::new())
        .with(ConstrainedObject)
        .with(local)
        .with(Collider::Ship)
        .with(bounding_volume)
        .build();
}

fn initialize_score(world: &mut World, game: &Game) {
    let font = world.read_resource::<Loader>().load(
        "font/square.ttf",
        TtfFormat,
        Default::default(),
        (),
        &world.read_resource(),
    );

    let score_transform = UiTransform::new(
        "Score".to_string(),
        Anchor::TopMiddle,
        0.,
        -50.,
        1.,
        200.,
        50.,
        0,
    );

    let score_text = world
        .create_entity()
        .with(score_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [1.0, 1.0, 1.0, 1.0],
            50.,
        )).build();

    let mods_transform = UiTransform::new(
        "Mods".to_string(),
        Anchor::TopRight,
        -200.,
        -50.,
        1.,
        200.,
        50.,
        0,
    );

    let modifiers_text = world
        .create_entity()
        .with(mods_transform)
        .with(UiText::new(
            font.clone(),
            game.modifiers.as_text(),
            [1.0, 0.0, 0.0, 1.0],
            20.,
        )).build();

    world.add_resource(Score {
        score_text,
        asteroids: 0,
        modifiers_text,
        current_modifiers: game.modifiers,
    });
}

pub struct PausedState;

impl<E> StateCallback<State, E> for PausedState {
    fn on_start(&mut self, _: &mut World) {
        println!("Game Paused");
    }

    fn update(&mut self, world: &mut World) -> Trans<State> {
        let Game { ref mut pause, .. } = *world.write_resource::<Game>();

        if *pause {
            *pause = false;
            return Trans::Pop;
        }

        Trans::None
    }
}
