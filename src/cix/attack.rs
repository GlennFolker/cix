use bevy::{
    prelude::*,
    sprite::Anchor,
};
use bevy_rapier2d::prelude::*;

use crate::{
    ext::*,
    GROUP_CIX, GROUP_BULLET, GROUP_STOP_PIERCE,
    CixSprites, GameAtlas,
    Cix, CixDirection,
    CixArm, CixArmTarget,
    CixAction, CixActState,
    Health, Timed,
};

use std::ops::RangeInclusive as RangeIncl;

#[derive(Component, Copy, Clone, Default)]
pub struct CixAttack {
    pub init: f64,
    pub at: Vec2,
}

#[derive(Component, Copy, Clone, Default)]
pub struct CixAttackState {
    pub shoot: f64,
}

#[derive(Component)]
pub struct CixLaser;

impl CixLaser {
    pub const ARM_DISTANCE: f32 = 56.;

    pub const LEN: f32 = 480.;
    pub const LIFE: f64 = 0.32;
    pub const WIDTH: f32 = 16.;
    pub const CAP_LENGTH: f32 = 64.;

    pub const COLOR: RangeIncl<Color> = Color::rgba(0.6, 1.2, 1.8, 1.)..=Color::rgba(0., 0.4, 1., 0.);

    pub const CHARGE: f64 = 0.8;
    pub const DAMAGE: f32 = 30.;
}

pub fn cix_attack_sys(
    mut commands: Commands,
    context: Res<RapierContext>, time: Res<Time>,
    mut cix: Query<(&CixActState, &CixAttack, &mut CixAttackState, &CixDirection, &GlobalTransform)>,
    mut arms: Query<(&mut CixArmTarget, &GlobalTransform)>,
    mut enemies: Query<&mut Health, Without<Cix>>,
    groups: Query<&CollisionGroups>,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<CixSprites>, atlas: Res<GameAtlas>,
) {
    let (input, &attack, mut state, &dir, &global_trns) = cix.single_mut();
    if input.pressed(CixAction::Attack) {
        if input.just_pressed(CixAction::Attack) {
            state.shoot = attack.init;
        }

        let mut prog = dir.progress;
        prog = prog * prog * (3. - 2. * prog);

        let p = if dir.right { prog } else { 1. - prog };
        let angle = Vec2::X
            .angle_between(attack.at)
            .angle_clamp_range(
                if p >= 0.5 { 0. } else { f32::PI },
                (90f32 + (1. - (p * 2. - 1.).abs()) * 10f32).to_radians(),
            );

        let ray_dir = Vec2::from_angle(angle);
        let ray_pos =
            global_trns.translation().truncate() + CixArm::TARGET_POINT +
            ray_dir * CixLaser::ARM_DISTANCE;

        for (mut arm, &arm_global_trns) in &mut arms {
            **arm = Some(ray_pos - arm_global_trns.translation().truncate());
        }

        let current = time.elapsed_seconds_f64();
        if current - state.shoot >= CixLaser::CHARGE {
            let mut toi = None;
            context.intersections_with_ray(
                ray_pos, ray_dir, CixLaser::LEN, true, QueryFilter::new().groups(CollisionGroups::new(GROUP_BULLET, !GROUP_CIX)),
                |e, intersect| {
                    if let Ok(&group) = groups.get(e) && group.memberships.contains(GROUP_STOP_PIERCE)  {
                        toi = Some(intersect.toi);
                        false
                    } else {
                        if let Ok(mut health) = enemies.get_mut(e) {
                            health.amount -= CixLaser::DAMAGE;
                        }

                        true
                    }
                },
            );

            let len = toi.unwrap_or(CixLaser::LEN);
            let start = ray_pos;

            commands.spawn((
                CixLaser,
                Timed::new(CixLaser::LIFE),
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: atlas.index(&atlases, &sprites.laser_end),
                        anchor: Anchor::Custom(Vec2::new(0.5, 0.)),
                        custom_size: Some(Vec2::new(CixLaser::WIDTH, CixLaser::WIDTH)),
                        color: *CixLaser::COLOR.start(),
                        ..default()
                    },
                    texture_atlas: atlas.clone_weak(),
                    transform: Transform::from_translation(start.extend(60.))
                        .with_rotation(Quat::from_axis_angle(Vec3::Z, angle)),
                    ..default()
                },
            )).with_children(|builder| { builder.spawn(
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: atlas.index(&atlases, &sprites.laser),
                        anchor: Anchor::Custom(Vec2::new(-0.5, 0.)),
                        custom_size: Some(Vec2::new((len - CixLaser::CAP_LENGTH / 2.).max(0.) + 1., CixLaser::WIDTH)),
                        color: *CixLaser::COLOR.start(),
                        ..default()
                    },
                    texture_atlas: atlas.clone_weak(),
                    transform: Transform::from_xyz(-0.5, 0., 0.),
                    ..default()
                },
            ).with_children(|builder| { builder.spawn(
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: atlas.index(&atlases, &sprites.laser_end),
                        anchor: Anchor::Custom(Vec2::new(0.5, 0.)),
                        custom_size: Some(Vec2::new(CixLaser::CAP_LENGTH, CixLaser::WIDTH)),
                        color: *CixLaser::COLOR.start(),
                        flip_y: true,
                        ..default()
                    },
                    texture_atlas: atlas.clone_weak(),
                    transform: Transform::from_xyz((len - CixLaser::CAP_LENGTH / 2.).max(0.) + 0.5, 0., 0.)
                        .with_rotation(Quat::from_axis_angle(Vec3::Z, f32::PI)),
                    ..default()
                },
            ); }); });

            state.shoot = current;
        }
    } else {
        for (mut arm, _) in &mut arms {
            **arm = None;
        }
    }
}

pub fn cix_attack_update_sys(
    mut sprite: Query<(Option<&Children>, &mut TextureAtlasSprite)>,
    mut laser: Query<(Entity, &Timed), With<CixLaser>>,
) {
    for (e, &timed) in &mut laser {
        let f = timed.fin();

        let mut target = e;
        while let Ok((children, mut sprite)) = sprite.get_mut(target) {
            sprite.color = CixLaser::COLOR.start().lerp(*CixLaser::COLOR.end(), 1. - (f - 1.) * (f - 1.));
            if let Some(custom_size) = sprite.custom_size.as_mut() {
                custom_size.y = CixLaser::WIDTH * (1. - f * f);
            }

            target = if let Some(children) = children {
                children[0]
            } else {
                break
            };
        }
    }
}
