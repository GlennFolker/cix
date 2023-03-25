use bevy::{
    prelude::*,
    ecs::system::SystemState,
};

use bevy_asset_loader::prelude::*;

use std::mem;

#[derive(AssetCollection, Resource)]
pub struct CixSprites {
    #[asset(path = "sprites/cix/head.png")]
    pub head: Handle<Image>,
    #[asset(path = "sprites/cix/particle.png")]
    pub particle: Handle<Image>,
    #[asset(path = "sprites/cix/eye.png")]
    pub eye: Handle<Image>,

    #[asset(path = "sprites/cix/red-collar.png")]
    pub red_collar: Handle<Image>,
    #[asset(path = "sprites/cix/blue-cape.png")]
    pub blue_cape: Handle<Image>,
    #[asset(path = "sprites/cix/pink-collar.png")]
    pub pink_collar: Handle<Image>,
    #[asset(path = "sprites/cix/red-scarf.png")]
    pub red_scarf: Handle<Image>,
    #[asset(path = "sprites/cix/pink-scarf.png")]
    pub pink_scarf: Handle<Image>,
}

#[derive(Resource, Deref)]
pub struct GameAtlas(pub Handle<TextureAtlas>);
impl FromWorld for GameAtlas {
    fn from_world(world: &mut World) -> Self {
        let (server, mut cix_sprites, mut images, mut atlases) = SystemState::<(
            Res<AssetServer>,
            ResMut<CixSprites>,
            ResMut<Assets<Image>>,
            ResMut<Assets<TextureAtlas>>,
        )>::new(world).get_mut(world);

        let mut builder = TextureAtlasBuilder::default();
        let mut add = |sprite: &mut Handle<Image>| {
            let handle = mem::replace(sprite, sprite.clone_weak());
            let image = images.get(&handle).expect(&format!("{:?} isn't an `Image` asset", server.get_handle_path(&handle)));
            builder.add_texture(handle, image);
        };

        add(&mut cix_sprites.head);
        add(&mut cix_sprites.particle);
        add(&mut cix_sprites.eye);

        add(&mut cix_sprites.red_collar);
        add(&mut cix_sprites.blue_cape);
        add(&mut cix_sprites.pink_collar);
        add(&mut cix_sprites.red_scarf);
        add(&mut cix_sprites.pink_scarf);

        let atlas = builder.finish(&mut images).expect("Couldn't build texture atlas");
        let atlas = atlases.add(atlas);

        Self(atlas)
    }
}

impl GameAtlas {
    #[inline]
    pub fn rect(&self, atlases: &Assets<TextureAtlas>, sprite: &Handle<Image>) -> Rect {
        let atlas = atlases.get(self).expect("Texture atlas deallocated");
        atlas.textures[atlas.get_texture_index(sprite).expect("Invalid texture atlas sprite")]
    }

    #[inline]
    pub fn index(&self, atlases: &Assets<TextureAtlas>, sprite: &Handle<Image>) -> usize {
        let atlas = atlases.get(self).expect("Texture atlas deallocated");
        atlas.get_texture_index(sprite).expect("Invalid texture atlas sprite")
    }

    #[inline]
    pub fn rect_index(&self, atlases: &Assets<TextureAtlas>, sprite: &Handle<Image>) -> (Rect, usize) {
        let atlas = atlases.get(self).expect("Texture atlas deallocated");
        let index = atlas.get_texture_index(sprite).expect("Invalid texture atlas sprite");
        (atlas.textures[index], index)
    }
}
