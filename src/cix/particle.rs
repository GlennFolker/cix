use bevy::prelude::*;

use rand::{
    prelude::*,
    distributions::Uniform,
};

use crate::{
    ColorExt as _,
    CixSprites, GameAtlas,
    Cix, Timed,
};

use std::ops::RangeInclusive as RangeIncl;

pub const CIX_PARTICLE_COLOR: RangeIncl<Color> = Color::rgba(0.2, 1.3, 2.5, 0.2)..=Color::rgba(0.3, 0.5, 2., 0.1);
pub const CIX_PARTICLE_ALPHA: RangeIncl<f32> = 0.75f32..=1f32;
pub const CIX_PARTICLE_COUNT: RangeIncl<u32> = 1..=3;
pub const CIX_PARTICLE_LIFE: RangeIncl<f64> = 0.25f64..=0.5f64;
pub const CIX_PARTICLE_RADIUS: RangeIncl<f32> = 3f32..=5f32;

#[derive(Component)]
pub struct CixParticle {
    pub alpha: f32,
    pub radius: f32,
}

pub fn cix_spawn_particle_sys(
    mut commands: Commands, time: Res<Time>,
    cix: Query<(Entity, &TextureAtlasSprite), With<Cix>>,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<CixSprites>, atlas: Res<GameAtlas>,
) {
    let mut rng = thread_rng();
    let angle_rng = Uniform::from(0f32..(360f32.to_radians()));
    let dst_rng = Uniform::from(0.3f32..=1f32);
    let alpha_rng = Uniform::from(CIX_PARTICLE_ALPHA);
    let lifetime_rng = Uniform::from(CIX_PARTICLE_LIFE);
    let radius_rng = Uniform::from(CIX_PARTICLE_RADIUS);

    let (cix, sprite) = cix.single();
    commands.entity(cix).with_children(|builder| {
        for _ in 0..((rng.gen_range(CIX_PARTICLE_COUNT) as f32 * time.delta_seconds() * 60.) as u32) {
            let (sin, cos) = angle_rng.sample(&mut rng).sin_cos();
            let radius = radius_rng.sample(&mut rng);

            let mut dst = 1. - dst_rng.sample(&mut rng);
            dst = 1. - dst * dst;

            let alpha = alpha_rng.sample(&mut rng);
            let r = (sprite.custom_size.unwrap().x / 2. - radius / 2.) * dst;

            builder.spawn((
                CixParticle { alpha, radius, },
                Timed {
                    life: 0.,
                    lifetime: lifetime_rng.sample(&mut rng),
                },
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: CIX_PARTICLE_COLOR.start().with_a(CIX_PARTICLE_COLOR.start().a() * alpha),
                        index: atlas.get(&atlases, &sprites.particle),
                        custom_size: Some(Vec2::splat(radius * 2.)),
                        ..default()
                    },
                    texture_atlas: atlas.clone_weak(),
                    transform: Transform::from_xyz(cos * r, sin * r, 1.),
                    ..default()
                },
            ));
        };
    });
}

pub fn cix_update_particle_sys(mut particles: Query<(&CixParticle, &Timed, &mut TextureAtlasSprite)>) {
    for (particle, timed, mut sprite) in &mut particles {
        let f = timed.fin();
        let col = f * f;
        let rad = 1. - f * f;

        let color = &mut sprite.color;
        *color = CIX_PARTICLE_COLOR.start().lerp(*CIX_PARTICLE_COLOR.end(), col);
        color.set_a(color.a() * particle.alpha);

        sprite.custom_size = Some(Vec2::splat(particle.radius * 2. * rad));
    }
}
