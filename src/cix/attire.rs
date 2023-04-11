use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    CixSprites, GameAtlas,
    CixDirection,
};

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum CixAttire {
    RedCollar,
    BlueCape,
    PinkCollar,
    RedScarf,
    PinkScarf,
}

impl CixAttire {
    pub const ALL: &[Self] = &[
        Self::RedCollar,
        Self::BlueCape,
        Self::PinkCollar,
        Self::RedScarf,
        Self::PinkScarf,
    ];

    pub const OFFSET: f32 = 27.;
    pub const ROTATE_SHRINK: f32 = 0.4;

    #[inline]
    pub fn sprite(self, sprites: &CixSprites) -> &Handle<Image> {
        use CixAttire::*;
        match self {
            RedCollar => &sprites.red_collar,
            BlueCape => &sprites.blue_cape,
            PinkCollar => &sprites.pink_collar,
            RedScarf => &sprites.red_scarf,
            PinkScarf => &sprites.pink_scarf,
        }
    }

    #[inline]
    pub fn offset(self) -> Vec2 {
        use CixAttire::*;
        match self {
            RedCollar => Vec2::new(0., 0.),
            BlueCape => Vec2::new(-14., -12.),
            PinkCollar => Vec2::new(1., 0.),
            RedScarf => Vec2::new(14., -26.),
            PinkScarf => Vec2::new(8., -25.),
        }
    }

    #[inline]
    pub fn height(self) -> f32 {
        use CixAttire::*;
        match self {
            RedCollar => 16.,
            BlueCape => 26.,
            PinkCollar => 13.,
            RedScarf => 55.,
            PinkScarf => 36.,
        }
    }
}

pub fn cix_direct_attire_sys(
    cix: Query<&CixDirection, Changed<CixDirection>>,
    mut attires: Query<(&CixAttire, &mut Transform, &mut TextureAtlasSprite)>,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<CixSprites>, atlas: Res<GameAtlas>,
) {
    let Ok(&dir) = cix.get_single() else { return };
    let mut prog = dir.progress;
    prog = prog * prog * (3. - 2. * prog);

    let anchor_prog = (prog * 2. - 1.) * if dir.right { 1. } else { -1. };
    let size_prog = 1. - (0.5 - (prog - 0.5).abs()) * 2. * CixAttire::ROTATE_SHRINK;
    let flip = if dir.right { prog } else { 1. - prog } < 0.5;

    for (&attire, mut trns, mut sprite) in &mut attires {
        let offset = attire.offset().x;
        trns.translation.x = offset * anchor_prog;

        let sprite_size = atlas.rect(&atlases, attire.sprite(&sprites)).size();
        sprite.custom_size = Some(Vec2::new(
            sprite_size.x * size_prog,
            sprite_size.y,
        ));
        sprite.flip_x = flip;
    }
}
