use bevy::prelude::*;
use rand::{
    prelude::*,
    distributions::Uniform,
};

use crate::{
    ext::*,
    GenericSprites, CixSprites, GameAtlas,
    CixStates,
    Timed,
};

#[derive(Component)]
pub struct CixSpawn;
impl CixSpawn {
    pub const TIME: f64 = 1.2;
    pub const RESPAWN_TIME: f64 = 2.;
}

#[derive(Component, Copy, Clone)]
pub struct CixSpawnParticle {
    pub offset: Vec2,
    pub radius: f32,
}

#[derive(Resource, Deref, DerefMut, Copy, Clone)]
pub struct CixSpawnPos(pub Vec2);

pub fn cix_init_spawn_sys(
    mut commands: Commands, pos: Res<CixSpawnPos>,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<GenericSprites>, atlas: Res<GameAtlas>,
) {
    let mut rng = thread_rng();
    let angle = Uniform::from(0f32..(360f32).to_radians());
    let dist = Uniform::from(160f32..=320f32);
    let radius = Uniform::from(2f32..=4f32);

    commands.spawn((
        (
            CixSpawn,
            Timed::new(CixSpawn::TIME),
        ),
        SpatialBundle::from(Transform::from_translation(pos.extend(50.))),
    )).with_children(|builder| {
        for _ in 0..48 {
            let offset = Vec2::from_angle(angle.sample(&mut rng)) * dist.sample(&mut rng);
            builder.spawn((
                CixSpawnParticle {
                    offset,
                    radius: radius.sample(&mut rng),
                },
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: atlas.index(&atlases, &sprites.circle),
                        color: Color::NONE,
                        custom_size: Some(Vec2::splat(0.)),
                        ..default()
                    },
                    texture_atlas: atlas.clone_weak(),
                    transform: Transform::from_translation(offset.extend(0.)),
                    ..default()
                },
            ));
        }
    });
}

pub fn cix_update_spawn_sys(
    mut commands: Commands, mut state: ResMut<NextState<CixStates>>,
    spawn: Query<(&Timed, &GlobalTransform), With<CixSpawn>>,
    mut particles: Query<(&CixSpawnParticle, &mut Transform, &mut TextureAtlasSprite)>,
    atlases: Res<Assets<TextureAtlas>>,
    generic_sprites: Res<GenericSprites>, cix_sprites: Res<CixSprites>, atlas: Res<GameAtlas>,
) {
    let (&timed, &global_transform) = spawn.single();
    let f = timed.fin();

    for (&particle, mut trns, mut sprite) in &mut particles {
        trns.translation = particle.offset.lerp(Vec2::splat(0.), f * f).extend(0.);
        sprite.color = Color::NONE.lerp(Color::rgba(0.4, 1.8, 3., 0.36), f);
        sprite.custom_size = Some(Vec2::splat(0f32.lerp(particle.radius * 2., f)));
    }

    if timed.ended() {
        crate::cix_spawn(&mut commands, &atlases, &generic_sprites, &cix_sprites, &atlas, global_transform);
        state.set(CixStates::Alive);
    }
}
