use amethyst::{
    ecs::{prelude::Entity, World},
    renderer::SpriteRender,
};

use crate::{components::Bounded, textures::SpriteSheet};

pub struct Ships {
    pub sprite_sheet: SpriteSheet,
}

impl Ships {
    pub fn initialize(world: &mut World) {
        let sprite_sheet = SpriteSheet::from_path(world, "texture/ship");
        world.add_resource(Ships { sprite_sheet });
    }

    pub fn new_sprite_render(&self) -> SpriteRender {
        self.sprite_sheet.sprite_render(0)
    }

    pub fn new_bounded(&self) -> Bounded {
        Bounded::from_local(6.0)
    }
}

pub struct Bullets {
    pub sprite_sheet: SpriteSheet,
}

impl Bullets {
    pub fn initialize(world: &mut World) {
        let sprite_sheet = SpriteSheet::from_path(world, "texture/bullet");
        world.add_resource(Bullets { sprite_sheet });
    }

    pub fn new_sprite_render(&self) -> SpriteRender {
        self.sprite_sheet.sprite_render(0)
    }

    pub fn new_bounded(&self) -> Bounded {
        Bounded::from_local(2.0)
    }
}

pub struct Asteroids {
    pub sprite_sheet: SpriteSheet,
}

impl Asteroids {
    pub const MIN_RADIUS: f32 = 4.0;
    pub const NUM_SPRITES: usize = 3;

    pub fn initialize(world: &mut World) {
        let sprite_sheet = SpriteSheet::from_path(world, "texture/asteroids");
        world.add_resource(Asteroids { sprite_sheet });
    }

    pub fn new_sprite_render(&self, random_gen: &RandomGen) -> SpriteRender {
        let index = random_gen.next_usize() % Self::NUM_SPRITES;
        self.sprite_sheet.sprite_render(index)
    }

    pub fn new_bounded(&self, scale: f32) -> Bounded {
        Bounded::from_local(Self::MIN_RADIUS * scale)
    }
}

pub struct RandomGen;

impl RandomGen {
    /// Generate a random usize.
    pub fn next_usize(&self) -> usize {
        use rand::Rng;
        rand::thread_rng().gen::<usize>()
    }

    pub fn next_f32(&self) -> f32 {
        use rand::Rng;
        rand::thread_rng().gen::<f32>()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GameModifiers {
    /// Player is immortal.
    pub player_is_immortal: bool,
    /// Player is dead.
    pub player_is_dead: bool,
}

impl GameModifiers {
    /// Get a text describing modifiers in place.
    pub fn as_text(&self) -> String {
        let mut list = Vec::new();

        if self.player_is_immortal {
            list.push("immortal (F2)");
        }

        if self.player_is_dead {
            list.push("dead (R to Restart)");
        }

        list.join(", ")
    }
}

#[derive(Default)]
pub struct Game {
    /// Restart the game.
    pub restart: bool,
    /// Pause the game.
    pub pause: bool,
    /// Game modifiers in place.
    pub modifiers: GameModifiers,
}

#[derive(Debug)]
pub struct Score {
    pub score_text: Entity,
    pub asteroids: u32,
    pub modifiers_text: Entity,
    pub current_modifiers: GameModifiers,
}
