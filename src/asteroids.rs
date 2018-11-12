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
    audio::initialise_audio,
    BoundingVolume, ConstrainedObject, Physical, Ship, ARENA_HEIGHT, ARENA_WIDTH,
};

#[derive(Default)]
pub struct Asteroids {
    pub player_is_immortal: bool,
}

impl<'a, 'b> SimpleState<'a, 'b> for Asteroids {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;

        ShipResource::initialize(world);
        BulletResource::initialize(world);
        AsteroidResource::initialize(world);
        world.add_resource(RandomGen);

        let game = {
            let mut game = GameResource::default();
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

    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans<'a, 'b> {
        let GameResource {
            restart,
            modifiers,
            ..
        } = *data.world.read_resource::<GameResource>();

        if restart {
            data.world.delete_all();

            return Trans::Switch(Box::new(
                Asteroids {
                    player_is_immortal: self.player_is_immortal || modifiers.player_is_immortal,
                }
            ));
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

fn initialize_score(world: &mut World, game: &GameResource) {
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
        0., -50., 1., 200., 50., 0,
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
        -200., -50., 1., 200., 50., 0,
    );

    let modifiers_text = world
        .create_entity()
        .with(mods_transform)
        .with(UiText::new(
            font.clone(),
            game.modifiers.as_text(),
            [1.0, 0.0, 0.0, 1.0], 20.,
        )).build();

    world.add_resource(Score {
        score_text,
        asteroids: 0,
        modifiers_text,
        current_modifiers: game.modifiers,
    });
}
