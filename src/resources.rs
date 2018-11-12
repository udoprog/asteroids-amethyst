use amethyst::{
    assets::{AssetStorage, Loader},
    ecs::{prelude::Entity, World},
    renderer::{
        MaterialTextureSet, PngFormat, SpriteRender, SpriteSheet, SpriteSheetFormat,
        SpriteSheetHandle, Texture, TextureMetadata,
    },
};

use crate::BoundingVolume;

pub struct ShipResource {
    pub sprite_sheet: SpriteSheetHandle,
}

impl ShipResource {
    pub fn initialize(world: &mut World) {
        let texture_handle = {
            let loader = world.read_resource::<Loader>();
            let texture_storage = world.read_resource::<AssetStorage<Texture>>();
            loader.load(
                "texture/ship.png",
                PngFormat,
                TextureMetadata::srgb_scale(),
                (),
                &texture_storage,
            )
        };

        let sprite_sheet = {
            let mut material_texture_set = world.write_resource::<MaterialTextureSet>();
            let texture_id = material_texture_set.len() as u64;
            material_texture_set.insert(texture_id, texture_handle);

            let loader = world.read_resource::<Loader>();
            let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
            loader.load(
                "texture/ship.ron",
                SpriteSheetFormat,
                texture_id,
                (),
                &sprite_sheet_store,
            )
        };

        world.add_resource(ShipResource { sprite_sheet });
    }

    pub fn new_sprite_render(&self) -> SpriteRender {
        SpriteRender {
            sprite_sheet: self.sprite_sheet.clone(),
            sprite_number: 0,
            flip_horizontal: false,
            flip_vertical: false,
        }
    }

    pub fn create_bounding_volume(&self, entity: Entity) -> BoundingVolume {
        BoundingVolume::from_local(entity, 6.0, Collider::Ship)
    }
}

pub struct BulletResource {
    pub sprite_sheet: SpriteSheetHandle,
}

impl BulletResource {
    pub fn initialize(world: &mut World) {
        let texture_handle = {
            let loader = world.read_resource::<Loader>();
            let texture_storage = world.read_resource::<AssetStorage<Texture>>();
            loader.load(
                "texture/bullet.png",
                PngFormat,
                TextureMetadata::srgb_scale(),
                (),
                &texture_storage,
            )
        };

        let sprite_sheet = {
            let mut material_texture_set = world.write_resource::<MaterialTextureSet>();
            let texture_id = material_texture_set.len() as u64;
            material_texture_set.insert(texture_id, texture_handle);

            let loader = world.read_resource::<Loader>();
            let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
            loader.load(
                "texture/bullet.ron",
                SpriteSheetFormat,
                texture_id,
                (),
                &sprite_sheet_store,
            )
        };

        world.add_resource(BulletResource { sprite_sheet });
    }

    pub fn new_sprite_render(&self) -> SpriteRender {
        SpriteRender {
            sprite_sheet: self.sprite_sheet.clone(),
            sprite_number: 0,
            flip_horizontal: false,
            flip_vertical: false,
        }
    }

    pub fn create_bounding_volume(&self, entity: Entity) -> BoundingVolume {
        BoundingVolume::from_local(entity, 2.0, Collider::Bullet)
    }
}

pub struct AsteroidResource {
    pub sprite_sheet: SpriteSheetHandle,
}

impl AsteroidResource {
    pub const MIN_RADIUS: f32 = 4.0;
    pub const NUM_SPRITES: usize = 3;

    pub fn initialize(world: &mut World) {
        let texture_handle = {
            let loader = world.read_resource::<Loader>();
            let texture_storage = world.read_resource::<AssetStorage<Texture>>();
            loader.load(
                "texture/asteroids.png",
                PngFormat,
                TextureMetadata::srgb_scale(),
                (),
                &texture_storage,
            )
        };

        let sprite_sheet = {
            let mut material_texture_set = world.write_resource::<MaterialTextureSet>();
            let texture_id = material_texture_set.len() as u64;
            material_texture_set.insert(texture_id, texture_handle);

            let loader = world.read_resource::<Loader>();
            let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
            loader.load(
                "texture/asteroids.ron",
                SpriteSheetFormat,
                texture_id,
                (),
                &sprite_sheet_store,
            )
        };

        world.add_resource(AsteroidResource { sprite_sheet });
    }

    pub fn new_sprite_render(&self, random_gen: &RandomGen) -> SpriteRender {
        let sprite_number = random_gen.next_usize() % Self::NUM_SPRITES;

        SpriteRender {
            sprite_sheet: self.sprite_sheet.clone(),
            sprite_number,
            flip_horizontal: false,
            flip_vertical: false,
        }
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

#[derive(Default)]
pub struct GameResource {
    pub player_is_dead: bool,
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
}
