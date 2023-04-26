use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

use crate::{
    ext::*,
    PIXELS_PER_METER,
    GenericSprites, GameAtlas,
    Cix, Timed,
};

use std::ops::RangeInclusive as RangeIncl;

#[derive(Component)]
pub struct CixFire {
    pub alpha: f32,
    pub radius: f32,
    pub velocity: Vec2,
}

impl CixFire {
    pub const COLOR: RangeIncl<Color> = Color::rgba(0.2, 1.3, 2.5, 0.3)..=Color::rgba(0., 0.2, 0.7, 0.);
    pub const ALPHA: RangeIncl<f32> = 0.5f32..=1f32;
    pub const CHANCE: f32 = 0.75;
    pub const LIFE: RangeIncl<f64> = 0.6f64..=1.5f64;
    pub const RADIUS: RangeIncl<f32> = 3f32..=7.2f32;
    pub const VELOCITY: RangeIncl<f32> = 0.3f32..=2.7f32;
}

pub fn cix_spawn_fire_sys(
    mut commands: Commands, time: Res<Time>,
    cix: Query<(&GlobalTransform, &TextureAtlasSprite, &Velocity), With<Cix>>,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<GenericSprites>, atlas: Res<GameAtlas>,
) {
    let mut rng = thread_rng();

    let Ok((&global_trns, sprite, &vel)) = cix.get_single() else { return };
    let trns = global_trns.translation();

    if rng.gen_range(0f32..=1f32) <= CixFire::CHANCE * time.delta_seconds() * 60. {
        let (sin, cos) = rng.gen_range(0f32..(180f32.to_radians())).sin_cos();
        let radius = rng.gen_range(CixFire::RADIUS);

        let scl = PIXELS_PER_METER;
        let velocity = Vec2::new(vel.linvel.x / scl, vel.linvel.y.max(0.) / scl + rng.gen_range(CixFire::VELOCITY));
        let alpha = rng.gen_range(CixFire::ALPHA);
        let rad = sprite.custom_size.unwrap().x / 2. - radius;

        commands.spawn((
            CixFire { alpha, radius, velocity, },
            Timed::new(rng.gen_range(CixFire::LIFE)),
            SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: CixFire::COLOR.start().with_a(CixFire::COLOR.start().a() * alpha),
                    index: atlas.index(&atlases, &sprites.circle),
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
    let delta = time.delta_seconds() * 60.;
    for (fire, timed, mut trns, mut sprite) in &mut fires {
        let f = timed.fin();
        let col = 1. - (f - 1.) * (f - 1.);
        let rad = 1. - f * f;

        let color = &mut sprite.color;
        *color = CixFire::COLOR.start().lerp(*CixFire::COLOR.end(), col);
        color.set_a(color.a() * fire.alpha);

        sprite.custom_size = Some(Vec2::splat(fire.radius * 2. * rad));
        trns.translation += (fire.velocity * (delta * (1. - f))).extend(0.);
    }
}
