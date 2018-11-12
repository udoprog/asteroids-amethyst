use amethyst::{
    ecs::{prelude::Entity, World},
    renderer::SpriteRender,
};

use crate::{
    textures::SpriteSheet,
    BoundingVolume,
};

pub struct ShipResource {
    pub sprite_sheet: SpriteSheet,
}

impl ShipResource {
    pub fn initialize(world: &mut World) {
        let sprite_sheet = SpriteSheet::from_path(world, "texture/ship");
        world.add_resource(ShipResource { sprite_sheet });
    }

    pub fn new_sprite_render(&self) -> SpriteRender {
        self.sprite_sheet.sprite_render(0)
    }

    pub fn create_bounding_volume(&self, entity: Entity) -> BoundingVolume {
        BoundingVolume::from_local(entity, 6.0, Collider::Ship)
    }
}

pub struct BulletResource {
    pub sprite_sheet: SpriteSheet,
}

impl BulletResource {
    pub fn initialize(world: &mut World) {
        let sprite_sheet = SpriteSheet::from_path(world, "texture/bullet");
        world.add_resource(BulletResource { sprite_sheet });
    }

    pub fn new_sprite_render(&self) -> SpriteRender {
        self.sprite_sheet.sprite_render(0)
    }

    pub fn create_bounding_volume(&self, entity: Entity) -> BoundingVolume {
        BoundingVolume::from_local(entity, 2.0, Collider::Bullet)
    }
}

pub struct AsteroidResource {
    pub sprite_sheet: SpriteSheet,
}

impl AsteroidResource {
    pub const MIN_RADIUS: f32 = 4.0;
    pub const NUM_SPRITES: usize = 3;

    pub fn initialize(world: &mut World) {
        let sprite_sheet = SpriteSheet::from_path(world, "texture/asteroids");
        world.add_resource(AsteroidResource { sprite_sheet });
    }

    pub fn new_sprite_render(&self, random_gen: &RandomGen) -> SpriteRender {
        let index = random_gen.next_usize() % Self::NUM_SPRITES;
        self.sprite_sheet.sprite_render(index)
    }

    pub fn create_bounding_volume(
        &self, entity: Entity, scale: f32, collider: impl Fn(Entity) -> Collider
    ) -> BoundingVolume {
        BoundingVolume::from_local(entity, Self::MIN_RADIUS * scale, collider)
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
pub struct GameResource {
    /// Restart the game.
    pub restart: bool,
    /// Game modifiers in place.
    pub modifiers: GameModifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Collider {
    Bullet(Entity),
    Ship(Entity),
    Asteroid(Entity),
    /// Asteroid can collide, but will not register collissions until it's gone one frame without
    /// collisions.
    DeferredAsteroid(Entity),
}

impl Collider {
    /// Access the entity this collider is part of.
    pub fn entity(&self) -> Entity {
        use self::Collider::*;

        match *self {
            Bullet(e) => e,
            Ship(e) => e,
            Asteroid(e) => e,
            DeferredAsteroid(e) => e,
        }
    }
}

#[derive(Debug)]
pub struct Score {
    pub score_text: Entity,
    pub asteroids: u32,
    pub modifiers_text: Entity,
    pub current_modifiers: GameModifiers,
}
