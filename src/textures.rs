use amethyst::{
    assets::{AssetStorage, Loader},
    ecs::World,
    renderer::{
        self, MaterialTextureSet, PngFormat, SpriteRender, SpriteSheetFormat, SpriteSheetHandle,
        Texture, TextureMetadata,
    },
};

/// A handle for a sprite sheet.
pub struct SpriteSheet {
    /// Handle to the sprite shit.
    pub handle: SpriteSheetHandle,
}

impl SpriteSheet {
    /// Load a sprite sheet from the given path, expecting a <path>.ron file for the mapping and a
    /// <path>.png file for the texture.
    pub fn from_path(world: &mut World, path: &str) -> SpriteSheet {
        let texture_handle = {
            let loader = world.read_resource::<Loader>();
            let texture_storage = world.read_resource::<AssetStorage<Texture>>();
            loader.load(
                format!("{}.png", path).as_str(),
                PngFormat,
                TextureMetadata::srgb_scale(),
                (),
                &texture_storage,
            )
        };

        let handle = {
            let mut material_texture_set = world.write_resource::<MaterialTextureSet>();
            let texture_id = material_texture_set.len() as u64;
            material_texture_set.insert(texture_id, texture_handle);

            let loader = world.read_resource::<Loader>();
            let sprite_sheet_store = world.read_resource::<AssetStorage<renderer::SpriteSheet>>();
            let handle = loader.load(
                format!("{}.ron", path).as_str(),
                SpriteSheetFormat,
                texture_id,
                (),
                &sprite_sheet_store,
            );

            handle
        };

        SpriteSheet { handle }
    }

    /// Construct a render handle for the given sprite in the sprite sheet.
    pub fn sprite_render(&self, sprite_number: usize) -> SpriteRender {
        SpriteRender {
            sprite_sheet: self.handle.clone(),
            sprite_number,
            flip_horizontal: false,
            flip_vertical: false,
        }
    }
}
