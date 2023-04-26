use bevy::{
    prelude::*,
    sprite::Anchor,
};
use bevy_rapier2d::prelude::*;
use rand::{
    prelude::*,
    distributions::Uniform,
};

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
    pub const ARM_DISTANCE: f32 = 60.;

    pub const LEN: f32 = 480.;
    pub const LIFE: f64 = 0.32;
    pub const WIDTH: f32 = 16.;
    pub const CAP_LENGTH: f32 = 64.;

    pub const COLOR: RangeIncl<Color> = Color::rgba(0.6, 1.2, 1.8, 1.)..=Color::rgba(0., 0.4, 1., 0.);

    pub const CHARGE: f64 = 0.8;
    pub const DAMAGE: f32 = 30.;
}

#[derive(Component, Copy, Clone, Default)]
pub struct CixLaserChargeParticle {
    pub offset: Vec2,
    pub radius: f32,
}

pub fn cix_attack_sys(
    mut commands: Commands,
    context: Res<RapierContext>, time: Res<Time>,
    mut cix: Query<(&CixActState, &CixAttack, &mut CixAttackState, &CixDirection, &GlobalTransform)>,
    mut charge_particles: Query<(&mut CixLaserChargeParticle, &mut Transform, &mut TextureAtlasSprite)>,
    mut arms: Query<(&mut CixArmTarget, &GlobalTransform)>,
    mut enemies: Query<&mut Health, Without<Cix>>, groups: Query<&CollisionGroups>,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<CixSprites>, atlas: Res<GameAtlas>,
) {
    let Ok((input, &attack, mut state, &dir, &global_trns)) = cix.get_single_mut() else { return };
    if input.pressed(CixAction::Attack) {
        let reset_charge = |charge_particles: &mut Query<(&mut CixLaserChargeParticle, &mut Transform, &mut TextureAtlasSprite)>| {
            let mut rng = thread_rng();
            let angle = Uniform::from(0f32..(360f32).to_radians());
            let dist = Uniform::from(48f32..=96f32);
            let radius = Uniform::from((CixLaser::WIDTH / 12.)..=(CixLaser::WIDTH / 2.));

            for (mut particle, mut trns, mut sprite) in charge_particles {
                particle.radius = radius.sample(&mut rng);
                particle.offset = Vec2::from_angle(angle.sample(&mut rng)) * dist.sample(&mut rng);
                trns.translation = particle.offset.extend(10.);
                sprite.color = *CixLaser::COLOR.end();
                sprite.custom_size = Some(Vec2::splat(0.));
            }
        };

        if input.just_pressed(CixAction::Attack) {
            state.shoot = attack.init;
            reset_charge(&mut charge_particles);
        }

        let mut prog = dir.progress;
        prog = prog * prog * (3. - 2. * prog);

        let p = if dir.right { prog } else { 1. - prog };
        let angle = Vec2::X
            .angle_between(attack.at)
            .angle_clamp_range(
                if p >= 0.5 { 0. } else { f32::PI },
                (90f32 + (1. - (p * 2. - 1.).abs()) * 30f32).to_radians(),
            );

        let pos = global_trns.translation().truncate();
        let ray_dir = Vec2::from_angle(angle);
        let ray_pos = pos + CixArm::TARGET_POINT + ray_dir * CixLaser::ARM_DISTANCE;

        for (mut arm, &arm_global_trns) in &mut arms {
            **arm = Some(ray_pos - arm_global_trns.translation().truncate());
        }

        let current = time.elapsed_seconds_f64();
        let f = ((current - state.shoot) / CixLaser::CHARGE).clamp(0., 1.) as f32;
        for (particle, mut trns, mut sprite) in &mut charge_particles {
            trns.translation = (particle.offset.lerp(Vec2::splat(0.), f * f * f) + ray_pos - pos).extend(10.);
            sprite.color = CixLaser::COLOR.end().lerp(*CixLaser::COLOR.start(), f);
            sprite.custom_size = Some(Vec2::splat(0.).lerp(Vec2::splat(particle.radius * 2.), 1. - (f - 1.) * (f - 1.)));
        }

        if current - state.shoot >= CixLaser::CHARGE {
            let mut hit = Vec::new();
            context.intersections_with_ray(
                ray_pos, ray_dir, CixLaser::LEN, true, QueryFilter::new().groups(CollisionGroups::new(GROUP_BULLET, !GROUP_CIX)),
                |e, intersect| {
                    hit.push((e, intersect.toi));
                    true
                },
            );

            hit.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            let mut stop = None;
            for (index, &(e, toi)) in hit.iter().enumerate() {
                if let Ok(group) = groups.get(e) && group.memberships.contains(GROUP_STOP_PIERCE) {
                    stop = Some((index, toi));
                    break;
                }
            }

            let (end, len) = stop.unwrap_or((hit.len(), CixLaser::LEN));
            for &(e, _) in &hit[0..end] {
                if let Ok(mut health) = enemies.get_mut(e) {
                    health.amount -= CixLaser::DAMAGE;
                }
            }

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
                    transform: Transform::from_translation(ray_pos.extend(60.))
                        .with_rotation(Quat::from_axis_angle(Vec3::Z, angle)),
                    ..default()
                },
            )).with_children(|builder| { builder.spawn(
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: atlas.index(&atlases, &sprites.laser),
                        anchor: Anchor::Custom(Vec2::new(-0.5, 0.)),
                        custom_size: Some(Vec2::new(len + 1., CixLaser::WIDTH)),
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
                    transform: Transform::from_xyz(len + 0.5, 0., 0.)
                        .with_rotation(Quat::from_axis_angle(Vec3::Z, f32::PI)),
                    ..default()
                },
            ); }); });

            state.shoot = current;
            reset_charge(&mut charge_particles);
        }
    } else {
        for (mut arm, _) in &mut arms {
            **arm = None;
        }
    }

    if input.just_released(CixAction::Attack) {
        for (_, _, mut sprite) in &mut charge_particles {
            sprite.color = *CixLaser::COLOR.end();
            sprite.custom_size = Some(Vec2::splat(0.));
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
