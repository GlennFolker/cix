use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::{
    prelude::*,
    distributions::Uniform,
};

use crate::{
    ext::*,
    GROUP_STATIC, GROUP_STOP_PIERCE, GROUP_GROUND,
    GenericSprites, StaticEnemySprites, GameAtlas,
    Timed,
};

use std::ops::RangeInclusive as RangeIncl;

#[derive(Component, Copy, Clone)]
pub struct EnemyBarrier {
    pub height: f32,
    pub color: Color,
}

impl EnemyBarrier {
    pub const CHANCE: f32 = 0.24;
    pub const RADIUS: RangeIncl<f32> = 3f32..=7.2f32;
    pub const TIME: f64 = 1.8;
}

#[derive(Component, Copy, Clone)]
pub struct EnemyBarrierParticle {
    pub init: f32,
    pub height: f32,
}

pub fn spawn_enemy_barrier(
    commands: &mut Commands,
    height: f32, color: Color,
    pos: Vec2,
    atlases: &Assets<TextureAtlas>,
    enemy_sprites: &StaticEnemySprites, atlas: &GameAtlas,
) {
    let height = height * 32.;
    commands.spawn((
        EnemyBarrier { height, color, },
        (
            RigidBody::Fixed,
            CollisionGroups::new(GROUP_STOP_PIERCE | GROUP_GROUND, Group::ALL),
            Collider::cuboid(16., 16.),
        ),
        SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(atlas.index(atlases, &enemy_sprites.barrier)),
            texture_atlas: atlas.clone_weak(),
            transform: Transform::from_translation(pos.extend(10.)),
            ..default()
        },
    )).with_children(|builder| { builder.spawn((
        Collider::cuboid(6., height / 2.),
        CollisionGroups::new(GROUP_STATIC, Group::ALL),
        TransformBundle::from(Transform::from_xyz(0., height / 2. + 16., 0.)),
    )); });
}

pub fn enemy_barrier_update_sys(
    mut commands: Commands, time: Res<Time>,
    barriers: Query<(&EnemyBarrier, &GlobalTransform)>,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<GenericSprites>, atlas: Res<GameAtlas>,
){
    let mut rng = thread_rng();
    let chance = Uniform::from(0f32..=1f32);
    let height = Uniform::from(0.75f32..=1f32);
    let start = Uniform::from(-16f32..=16f32);
    let radius = Uniform::from(EnemyBarrier::RADIUS);

    for (&barrier, &global_trns) in &barriers {
        if chance.sample(&mut rng) <= EnemyBarrier::CHANCE * time.delta_seconds() * 60. {
            let pos = global_trns.translation();
            commands.spawn((
                EnemyBarrierParticle {
                    init: pos.y,
                    height: barrier.height * height.sample(&mut rng) + 32.,
                },
                Timed::new(EnemyBarrier::TIME * (barrier.height as f64 / 320.)),
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: atlas.index(&atlases, &sprites.circle),
                        color: barrier.color,
                        custom_size: Some(Vec2::splat(radius.sample(&mut rng) * 2.)),
                        ..default()
                    },
                    texture_atlas: atlas.clone_weak(),
                    transform: Transform::from_xyz(pos.x + start.sample(&mut rng), pos.y + 16., pos.z + 5.),
                    ..default()
                },
            ));
        }
    }
}

pub fn enemy_barrier_particle_update_sys(mut particles: Query<(&EnemyBarrierParticle, &Timed, &mut Transform, &mut TextureAtlasSprite)>) {
    for (&particle, &timed, mut trns, mut sprite) in &mut particles {
        let f = timed.fin();
        trns.translation.y = particle.init.lerp(particle.init + particle.height, 1. - (f - 1.) * (f - 1.));
        sprite.color.set_a((f - 1.) * (f - 1.));
    }
}
