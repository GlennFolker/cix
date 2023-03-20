use bevy::prelude::*;

use crate::{
    ColorExt as _,
    Cix,
};

use std::ops::RangeInclusive as RangeIncl;

pub const CIX_COLOR: RangeIncl<Color> = Color::rgba(0.2, 1.4, 2.5, 0.3)..=Color::rgba(0.4, 1.8, 3., 0.36);
pub const CIX_WAVE_SCALE: f32 = 16.;
pub const CIX_RADIUS: RangeIncl<f32> = 24f32..=26f32;

pub fn cix_update_head_sys(
    time: Res<Time>,
    mut cix: Query<&mut TextureAtlasSprite, With<Cix>>,
) {
    let absin = (time.elapsed_seconds() * CIX_WAVE_SCALE).sin() / 2. + 0.5;

    let mut sprite = cix.single_mut();
    sprite.color = CIX_COLOR.start().lerp(*CIX_COLOR.end(), absin);
    sprite.custom_size = Some(Vec2::splat((CIX_RADIUS.start() + absin * (CIX_RADIUS.end() - CIX_RADIUS.start())) * 2.));
}
