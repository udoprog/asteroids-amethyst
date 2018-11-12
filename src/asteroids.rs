use amethyst::{
    assets::Loader,
    core::transform::Transform,
    ecs::prelude::World,
    prelude::*,
    renderer::{Camera, Projection},
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};

use crate::{
    resources::{AsteroidResource, BulletResource, GameResource, RandomGen, Score, ShipResource},
    BoundingVolume, ConstrainedObject, Physical, Ship, ARENA_HEIGHT, ARENA_WIDTH,
};

pub struct Asteroids;

impl<'a, 'b> SimpleState<'a, 'b> for Asteroids {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;

        ShipResource::initialize(world);
        BulletResource::initialize(world);
        AsteroidResource::initialize(world);
        world.add_resource(RandomGen);
        world.add_resource(GameResource::default());

        initialize_score(world);

        // Setup our game.
        initialise_ship(world);
        initialise_camera(world);
    }

    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans<'a, 'b> {
        let game_resource = data.world.read_resource::<GameResource>();

        if game_resource.player_is_dead {
            return Trans::Quit;
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
        let ship_resource = world.read_resource::<ShipResource>();
        ship_resource.new_sprite_render()
    };

    let entity = world
        .create_entity()
        .with(sprite_render)
        .with(Ship::default())
        .with(Physical::new())
        .with(ConstrainedObject)
        .build();

    let bounding_volume = {
        let ship_resource = world.read_resource::<ShipResource>();
        ship_resource.create_bounding_volume(entity)
    };

    world
        .write_storage::<BoundingVolume>()
        .insert(entity, bounding_volume)
        .unwrap();
    world
        .write_storage::<Transform>()
        .insert(entity, local)
        .unwrap();
}

fn initialize_score(world: &mut World) {
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

    world.add_resource(Score {
        score_text,
        asteroids: 0,
    });
}
