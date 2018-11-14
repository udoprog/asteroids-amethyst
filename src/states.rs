use amethyst::{
    shred::{
        DispatcherBuilder, Dispatcher,
    },
    assets::Loader,
    core::{
        ArcThreadPool,
        transform::Transform,
    },
    ecs::prelude::World,
    prelude::{
        State, StateEvent, StateData, GameDataBuilder, GameData, Trans, Builder, DataInit,
    },
    renderer::{Camera, Projection},
    ui::{Anchor, TtfFormat, UiText, UiTransform},
    input::is_close_requested,
};

pub struct Data<'a, 'b> {
    // Base dispatcher.
    pub base: GameData<'a, 'b>,
    // Dispatcher for the main game.
    pub main: Dispatcher<'a, 'b>,
}

#[derive(Default)]
pub struct DataBuilder<'a, 'b> {
    pub base: GameDataBuilder<'a, 'b>,
    pub main: DispatcherBuilder<'a, 'b>,
}

impl<'a, 'b> DataInit<Data<'a, 'b>> for DataBuilder<'a, 'b> {
    fn build(self, world: &mut World) -> Data<'a, 'b> {
        let base = self.base.build(world);

        let mut main = {
            let pool = world.read_resource::<ArcThreadPool>();
            self.main.with_pool(pool.clone()).build()
        };

        main.setup(&mut world.res);

        Data {
            base,
            main,
        }
    }
}

type CustomTrans<'a, 'b> = Trans<Data<'a, 'b>, StateEvent>;

use crate::{
    audio::initialise_audio,
    components::{Collider, ConstrainedObject, Physical, Ship},
    resources::{Asteroids, Bullets, Game, RandomGen, Score, Ships},
    ARENA_HEIGHT, ARENA_WIDTH,
};

#[derive(Default)]
pub struct MainGameState {
    pub player_is_immortal: bool,
}

impl<'a, 'b> State<Data<'a, 'b>, StateEvent> for MainGameState {
    fn on_start(&mut self, data: StateData<Data>) {
        let StateData { world, .. } = data;

        Ships::initialize(world);
        Bullets::initialize(world);
        Asteroids::initialize(world);
        world.add_resource(RandomGen);

        let game = {
            let mut game = Game::default();
            game.modifiers.player_is_immortal = self.player_is_immortal;
            game
        };

        initialize_score(world, &game);

        world.add_resource(game);

        // Setup our game.
        initialise_ship(world);
        initialise_camera(world);
        initialise_audio(world);
    }

    fn update(&mut self, data: StateData<Data>) -> CustomTrans<'a, 'b> {
        let StateData {
            data,
            world,
            ..
        } = data;

        let Data {
            ref mut base,
            ref mut main,
        } = *data;

        base.update(world);
        main.dispatch(&world.res);

        let Game {
            restart, modifiers, ..
        } = *world.read_resource::<Game>();

        if restart {
            world.delete_all();

            return Trans::Switch(Box::new(MainGameState {
                player_is_immortal: self.player_is_immortal || modifiers.player_is_immortal,
            }));
        }

        let Game {
            ref mut pause, ..
        } = *world.write_resource::<Game>();

        if *pause {
            // NB: prevent a pause cycle by resetting the pause when acted on.
            *pause = false;
            return Trans::Push(Box::new(PauseState));
        }

        Trans::None
    }

    fn handle_event(
        &mut self,
        _data: StateData<Data>,
        event: StateEvent,
    ) -> CustomTrans<'a, 'b> {
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
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

/// State used when game is paused.
#[derive(Default)]
pub struct PauseState;

impl<'a, 'b> State<Data<'a, 'b>, StateEvent> for PauseState {
    fn on_start(&mut self, _: StateData<Data>) {
        println!("Game Paused");
    }

    fn update(&mut self, data: StateData<Data>) -> CustomTrans<'a, 'b> {
        let StateData {
            data,
            world,
            ..
        } = data;

        let Data {
            ref mut base,
            ..
        } = *data;

        base.update(world);

        let Game {
            ref mut pause, ..
        } = *world.write_resource::<Game>();

        if *pause {
            *pause = false;
            return Trans::Pop;
        }

        Trans::None
    }
}
