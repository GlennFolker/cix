use bevy::prelude::*;
use rand::{
    prelude::*,
    distributions::Uniform,
};

use crate::{
    ext::*,
    GenericSprites, GameAtlas,
    Cix, CixStates, CixSpawn, CixSpawnPos,
    CameraPos,
    Timed,
    DeathEvent,
};

#[derive(Component, Copy, Clone)]
pub enum CixDeathParticle {
    Blast,
    Large {
        init: Vec2,
        offset: Vec2,
        radius: f32,
    },
    Small {
        init: Vec2,
        offset: Vec2,
        radius: f32,
    },
}

pub fn cix_check_alive_sys(
    mut commands: Commands,
    mut state: ResMut<NextState<CixStates>>,
    mut events: EventReader<DeathEvent>,
    cix: Query<&GlobalTransform, With<Cix>>,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<GenericSprites>, atlas: Res<GameAtlas>,
) {
    for &DeathEvent(e) in &mut events {
        if let Ok(&global_trns) = cix.get(e) {
            state.set(CixStates::Dead);

            let mut rng = thread_rng();
            let angle = Uniform::from(0f32..(360f32).to_radians());
            let dist_large = Uniform::from(80f32..=160f32);
            let dist_small = Uniform::from(160f32..=320f32);
            let radius_large = Uniform::from(12f32..=24f32);
            let radius_small = Uniform::from(2f32..=12f32);
            let time_large = Uniform::from(1.2f64..=2f64);
            let time_small = Uniform::from(0.5f64..=1.3f64);
            let init = global_trns.translation().truncate();

            commands.spawn((
                CixDeathParticle::Blast,
                Timed::new(0.2),
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: atlas.index(&atlases, &sprites.circle),
                        color: Color::rgba(0.4, 1.8, 3., 0.36),
                        custom_size: Some(Vec2::splat(64.)),
                        ..default()
                    },
                    texture_atlas: atlas.clone_weak(),
                    transform: global_trns.into(),
                    ..default()
                },
            ));

            for _ in 0..8 {
                let offset = Vec2::from_angle(angle.sample(&mut rng)) * dist_large.sample(&mut rng);
                let radius = radius_large.sample(&mut rng);
                commands.spawn((
                    CixDeathParticle::Large { init, offset, radius, },
                    Timed::new(time_large.sample(&mut rng)),
                    SpriteSheetBundle {
                        sprite: TextureAtlasSprite {
                            index: atlas.index(&atlases, &sprites.circle),
                            color: Color::rgba(0.4, 1.8, 3., 0.36),
                            custom_size: Some(Vec2::splat(radius * 2.)),
                            ..default()
                        },
                        texture_atlas: atlas.clone_weak(),
                        transform: global_trns.into(),
                        ..default()
                    },
                ));
            }

            for _ in 0..32 {
                let offset = Vec2::from_angle(angle.sample(&mut rng)) * dist_small.sample(&mut rng);
                let radius = radius_small.sample(&mut rng);
                commands.spawn((
                    CixDeathParticle::Large { init, offset, radius, },
                    Timed::new(time_small.sample(&mut rng)),
                    SpriteSheetBundle {
                        sprite: TextureAtlasSprite {
                            index: atlas.index(&atlases, &sprites.circle),
                            color: Color::rgba(0.1, 0.6, 2., 0.24),
                            custom_size: Some(Vec2::splat(radius * 2.)),
                            ..default()
                        },
                        texture_atlas: atlas.clone_weak(),
                        transform: global_trns.into(),
                        ..default()
                    },
                ));
            }
        }
    }
}

pub fn cix_update_death_sys(mut particles: Query<(&CixDeathParticle, &Timed, &mut Transform, &mut TextureAtlasSprite)>) {
    for (&particle, &timed, mut trns, mut sprite) in &mut particles {
        let f = timed.fin();
        let (color_from, color_to) = match particle {
            CixDeathParticle::Blast | CixDeathParticle::Large { .. } => (Color::rgba(0.4, 1.8, 3., 0.36), Color::rgba(0., 0.4, 1., 0.)),
            CixDeathParticle::Small { .. } => (Color::rgba(0.1, 0.6, 2., 0.24), Color::rgba(0., 0.1, 0.5, 0.)),
        };

        match particle {
            CixDeathParticle::Blast => {
                sprite.color = color_from.lerp(color_to, (f - 1.) * (f - 1.) * (f - 1.) + 1.);
                sprite.custom_size = Some(Vec2::splat(64.).lerp(Vec2::splat(320.), 1. - (f - 1.) * (f - 1.)));
            },
            CixDeathParticle::Large { init, offset, radius, } | CixDeathParticle::Small { init, offset, radius, } => {
                trns.translation = (init + offset * (1. - (f - 1.) * (f - 1.))).extend(trns.translation.z);
                sprite.color = color_from.lerp(color_to, f * f);
                sprite.custom_size = Some(Vec2::splat(radius * 2. * (1. - f * f)));
            },
        }
    }
}

pub fn cix_respawn_sys(
    time: Res<Time>,
    mut state: ResMut<NextState<CixStates>>,
    mut camera_pos: ResMut<CameraPos>, cix_pos: Res<CixSpawnPos>,
    mut start: Local<Option<f64>>,
) {
    let current = time.elapsed_seconds_f64();
    if start.is_none() {
        *start = Some(current);
    }

    if current - start.unwrap() >= CixSpawn::RESPAWN_TIME {
        *start = None;

        **camera_pos = **cix_pos;
        state.set(CixStates::Spawning);
    }
}
