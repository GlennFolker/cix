use bevy::{
    prelude::*,
    ecs::system::SystemState,
    render::{
        render_resource::{
            SamplerDescriptor, AddressMode,
        },
        texture::ImageSampler,
    },
};

use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use std::mem;

pub const ATLAS_PAD: (usize, usize) = (4, 4);

#[derive(AssetCollection, Resource)]
pub struct Fonts {
    #[asset(path = "fonts/font.ttf")]
    pub font: Handle<Font>,
}

#[derive(AssetCollection, Resource, Deref, DerefMut)]
pub struct LdtkWorld {
    #[asset(path = "worlds/world.ldtk")]
    pub handle: Handle<LdtkAsset>,
}

#[derive(AssetCollection, Resource)]
pub struct BackgroundImages {
    #[asset(path = "worlds/background-back.png")]
    pub back: Handle<Image>,
    #[asset(path = "worlds/background-front.png")]
    pub front: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct GenericSprites {
    #[asset(path = "sprites/generic/circle.png")]
    pub circle: Handle<Image>,
    #[asset(path = "sprites/generic/square.png")]
    pub square: Handle<Image>,
    #[asset(path = "sprites/generic/triangle.png")]
    pub triangle: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct EnvironmentSprites {
    #[asset(path = "sprites/environment/gate.png")]
    pub gate: Handle<Image>,
    #[asset(path = "sprites/environment/petal.png")]
    pub petal: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct CixSprites {
    #[asset(path = "sprites/cix/head.png")]
    pub head: Handle<Image>,
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

    #[asset(path = "sprites/cix/arm-front-upper.png")]
    pub arm_front_upper: Handle<Image>,
    #[asset(path = "sprites/cix/arm-front-lower.png")]
    pub arm_front_lower: Handle<Image>,
    #[asset(path = "sprites/cix/arm-back-upper.png")]
    pub arm_back_upper: Handle<Image>,
    #[asset(path = "sprites/cix/arm-back-lower.png")]
    pub arm_back_lower: Handle<Image>,

    #[asset(path = "sprites/cix/laser.png")]
    pub laser: Handle<Image>,
    #[asset(path = "sprites/cix/laser-end.png")]
    pub laser_end: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct StaticEnemySprites {
    #[asset(path = "sprites/enemies/static/barrier.png")]
    pub barrier: Handle<Image>,
    #[asset(path = "sprites/enemies/static/gear.png")]
    pub gear: Handle<Image>,
}

#[derive(Resource, Deref)]
pub struct GameAtlas(pub Handle<TextureAtlas>);
impl FromWorld for GameAtlas {
    fn from_world(world: &mut World) -> Self {
        let (server, bg, generic_sprites, env_sprites, cix_sprites, enemy_static_sprites, mut images, mut atlases) = SystemState::<(
            Res<AssetServer>,
            ResMut<BackgroundImages>,
            ResMut<GenericSprites>,
            ResMut<EnvironmentSprites>,
            ResMut<CixSprites>,
            ResMut<StaticEnemySprites>,
            ResMut<Assets<Image>>,
            ResMut<Assets<TextureAtlas>>,
        )>::new(world).get_mut(world);

        let bg = bg.into_inner();
        for handle in [&mut bg.back, &mut bg.front] {
            let image = images.get_mut(handle).unwrap();
            image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                ..ImageSampler::linear_descriptor()
            });
        }

        let generic_sprites = generic_sprites.into_inner();
        let env_sprites = env_sprites.into_inner();
        let cix_sprites = cix_sprites.into_inner();
        let enemy_static_sprites = enemy_static_sprites.into_inner();

        let mut builder = TextureAtlasBuilder::default();

        let (pad_x, pad_y) = ATLAS_PAD;
        for handle in [
            &mut generic_sprites.circle,
            &mut generic_sprites.square,
            &mut generic_sprites.triangle,

            &mut env_sprites.gate,
            &mut env_sprites.petal,

            &mut cix_sprites.head,
            &mut cix_sprites.eye,

            &mut cix_sprites.red_collar,
            &mut cix_sprites.blue_cape,
            &mut cix_sprites.pink_collar,
            &mut cix_sprites.red_scarf,
            &mut cix_sprites.pink_scarf,

            &mut cix_sprites.arm_front_upper,
            &mut cix_sprites.arm_front_lower,
            &mut cix_sprites.arm_back_upper,
            &mut cix_sprites.arm_back_lower,

            &mut cix_sprites.laser,
            &mut cix_sprites.laser_end,

            &mut enemy_static_sprites.barrier,
            &mut enemy_static_sprites.gear,
        ] {
            let handle = mem::replace(handle, handle.clone_weak());
            let image = images.get_mut(&handle).unwrap_or_else(|| panic!("{:?} is deallocated", server.get_handle_path(&handle)));

            let pixel_size = 4 * mem::size_of::<u8>();
            let (width, height) = {
                let size = image.texture_descriptor.size;
                (size.width as usize, size.height as usize)
            };

            let (canvas_width, canvas_height) = (width + pad_x * 2, height + pad_y * 2);

            let canvas = &mut image.data;
            let frame = mem::replace(canvas, vec![0; canvas_width * canvas_height * pixel_size]);

            let row_len = width * pixel_size;
            for y in pad_y..(canvas_height - pad_y) {
                let frame_row = (y - pad_y) * row_len;
                let canvas_row = (y * canvas_width + pad_x) * pixel_size;
                canvas[canvas_row..(canvas_row + row_len)].copy_from_slice(&frame[frame_row..(frame_row + row_len)]);
            }

            {
                let size = &mut image.texture_descriptor.size;
                size.width = canvas_width as u32;
                size.height = canvas_height as u32;
            }

            builder.add_texture(handle, image);
        }

        let mut atlas = match builder.finish(&mut images) {
            Ok(atlas) => atlas,
            Err(e) => panic!("Couldn't build texture atlas: {e}"),
        };

        let pad = Vec2::new(pad_x as f32, pad_y as f32);
        for rect in &mut atlas.textures {
            rect.min += pad;
            rect.max -= pad;
        }

        Self(atlases.add(atlas))
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
