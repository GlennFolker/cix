use bevy::prelude::*;
use rand::prelude::*;

use crate::{
    ColorExt as _,
    CixSprites, GameAtlas,
    Cix, Timed,
};

use std::ops::RangeInclusive as RangeIncl;

pub const CIX_FIRE_COLOR: RangeIncl<Color> = Color::rgba(0.2, 1.3, 2.5, 0.3)..=Color::rgba(0., 0.2, 0.7, 0.);
pub const CIX_FIRE_ALPHA: RangeIncl<f32> = 0.5f32..=1f32;
pub const CIX_FIRE_CHANCE: f32 = 0.5;
pub const CIX_FIRE_LIFE: RangeIncl<f64> = 0.6f64..=1.5f64;
pub const CIX_FIRE_RADIUS: RangeIncl<f32> = 3f32..=7.2f32;
pub const CIX_FIRE_VELOCITY: RangeIncl<f64> = 0.3f64..=2.7f64;

#[derive(Component)]
pub struct CixFire {
    pub alpha: f32,
    pub radius: f32,
    pub velocity: f64,
}

pub fn cix_spawn_fire_sys(
    mut commands: Commands, time: Res<Time>,
    cix: Query<(&GlobalTransform, &TextureAtlasSprite), With<Cix>>,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<CixSprites>, atlas: Res<GameAtlas>,
) {
    let mut rng = thread_rng();

    let (global_trns, sprite) = cix.single();
    let trns = global_trns.translation();

    if rng.gen_range(0f32..=1f32) * time.delta_seconds() * 60. <= CIX_FIRE_CHANCE {
        let (sin, cos) = rng.gen_range(0f32..(180f32.to_radians())).sin_cos();
        let radius = rng.gen_range(CIX_FIRE_RADIUS);

        let velocity = rng.gen_range(CIX_FIRE_VELOCITY);
        let alpha = rng.gen_range(CIX_FIRE_ALPHA);
        let rad = sprite.custom_size.unwrap().x / 2. - radius;

        commands.spawn((
            CixFire { alpha, radius, velocity, },
            Timed {
                life: 0.,
                lifetime: rng.gen_range(CIX_FIRE_LIFE),
            },
            SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: CIX_FIRE_COLOR.start().with_a(CIX_FIRE_COLOR.start().a() * alpha),
                    index: atlas.get(&atlases, &sprites.particle),
                    custom_size: Some(Vec2::splat(radius * 2.)),
                    ..default()
                },
                texture_atlas: atlas.clone_weak(),
                transform: Transform::from_xyz(trns.x + cos * rad, trns.y + sin * rad, trns.z + 1.),
                ..default()
            },
        ));
    }
}

pub fn cix_update_fire_sys(
    time: Res<Time>,
    mut fires: Query<(&CixFire, &Timed, &mut Transform, &mut TextureAtlasSprite)>,
) {
    let delta = time.delta_seconds_f64() * 60.;
    for (fire, timed, mut trns, mut sprite) in &mut fires {
        let f = timed.fin();
        let col = 1. - (f - 1.) * (f - 1.);
        let rad = 1. - f * f;

        let color = &mut sprite.color;
        *color = CIX_FIRE_COLOR.start().lerp(*CIX_FIRE_COLOR.end(), col);
        color.set_a(color.a() * fire.alpha);

        sprite.custom_size = Some(Vec2::splat(fire.radius * 2. * rad));
        trns.translation.y += (fire.velocity * delta) as f32 * (1. - f);
    }
}
