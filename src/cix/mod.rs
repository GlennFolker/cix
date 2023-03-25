use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{
    GROUP_CIX,
    ColorExt as _,
    CixSprites, GameAtlas,
};

use std::ops::RangeInclusive as RangeIncl;

mod attire;
mod input;
mod particle;
mod fire;
mod eye;

pub use attire::*;
pub use input::*;
pub use particle::*;
pub use fire::*;
pub use eye::*;

#[derive(Component)]
pub struct Cix;
impl Cix {
    pub const COLOR: RangeIncl<Color> = Color::rgba(0.2, 1.4, 2.5, 0.3)..=Color::rgba(0.4, 1.8, 3., 0.36);
    pub const WAVE_SCALE: f32 = 16.;
    pub const RADIUS: RangeIncl<f32> = 24f32..=26f32;
    pub const HOVER_RAY: f32 = 100.;
}

#[derive(Component, Deref, DerefMut, Copy, Clone)]
pub struct CixGrounded(pub bool);

#[derive(Component, Copy, Clone)]
pub struct CixDirection {
    pub right: bool,
    pub progress: f32,
}

impl CixDirection {
    pub const TURN_SPEED: f32 = 3.2;
}

pub fn cix_spawn_sys(
    mut commands: Commands,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<CixSprites>, atlas: Res<GameAtlas>,
) {
    let group = CollisionGroups::new(GROUP_CIX, !GROUP_CIX);
    commands.spawn((
        (
            Cix,
            CixGrounded(false),
            CixDirection {
                right: true,
                progress: 1.,
            },
        ),
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                color: *Cix::COLOR.start(),
                index: atlas.index(&atlases, &sprites.head),
                custom_size: Some(Vec2::splat(Cix::RADIUS.start() * 2.)),
                ..default()
            },
            texture_atlas: atlas.clone_weak(),
            transform: Transform::from_xyz(0., 0., 50.),
            ..default()
        },
        (
            RigidBody::Dynamic,
            Collider::ball(*Cix::RADIUS.start()),
            group,
        ),
        (
            Velocity::default(),
            ExternalForce::default(),
            Damping {
                linear_damping: 0.5,
                angular_damping: 0.5,
            },
        ),
        (
            Sleeping::disabled(),
            Ccd::enabled(),
            LockedAxes::ROTATION_LOCKED_Z,
        ),
        InputManagerBundle::<CixAction> {
            action_state: default(),
            input_map: {
                let mut map = InputMap::default();
                map
                    .insert(VirtualDPad {
                        up: KeyCode::W.into(),
                        down: KeyCode::S.into(),
                        left: KeyCode::A.into(),
                        right: KeyCode::D.into(),
                    }, CixAction::Move)
                    .insert(KeyCode::Z, CixAction::Jump);

                map
            },
        },
    )).with_children(|builder| {
        builder.spawn((
            CixEye,
            SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: CixEye::COLOR,
                    index: atlas.index(&atlases, &sprites.eye),
                    ..default()
                },
                texture_atlas: atlas.clone_weak(),
                transform: Transform::from_xyz(0., 0., 2.),
                ..default()
            },
        ));

        for (i, &attire) in CixAttire::ALL.into_iter().enumerate() {
            let offset = attire.offset();
            let layer = 4. - (i as f32 / CixAttire::ALL.len() as f32);

            let (collider_w, collider_h) = attire.collider();
            let anchor1 = Vec2::new(offset.x, offset.y + collider_h / 2. - CixAttire::OFFSET);
            let anchor2 = Vec2::new(0., collider_h / 2.);

            builder.spawn((
                attire,
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite::new(atlas.index(&atlases, attire.sprite(&sprites))),
                    texture_atlas: atlas.clone_weak(),
                    transform: Transform::from_translation((anchor1 - anchor2).extend(layer)),
                    ..default()
                },
                (
                    RigidBody::Dynamic,
                    Collider::cuboid(collider_w / 2., collider_h / 2.),
                    group,
                    ImpulseJoint::new(builder.parent_entity(), {
                        let angle = attire.joint_angle() / 2.;
                        let mut joint = RevoluteJointBuilder::new()
                            .limits([-angle, angle])
                            .local_anchor1(anchor1)
                            .local_anchor2(anchor2)
                            .build();

                        joint.set_contacts_enabled(false);
                        joint
                    }),
                ),
                (
                    Sensor,
                    ColliderMassProperties::MassProperties(MassProperties {
                        local_center_of_mass: Vec2::new(0., collider_h / -2.),
                        mass: 0.015,
                        principal_inertia: 10.,
                    }),
                    Damping {
                        linear_damping: 5.,
                        angular_damping: 5.,
                    },
                ),
            ));
        }
    });
}

pub fn cix_pre_update_sys(mut cix: Query<&mut ExternalForce, With<Cix>>) {
    let mut force = cix.single_mut();
    *force = default();
}

pub fn cix_update_sys(
    context: Res<RapierContext>,
    mut cix: Query<(&mut CixGrounded, &GlobalTransform, &CollisionGroups, &Velocity, &mut ExternalForce), With<Cix>>,
) {
    let (mut grounded, &global_trns, &group, &vel, mut force) = cix.single_mut();

    let center = global_trns.translation().truncate();
    let ray_pos = Vec2::new(center.x, center.y - Cix::RADIUS.start());
    let ray_dir = Vec2::new(0., -1.);

    if let Some((_, toi)) = context.cast_ray(
        ray_pos, ray_dir,
        Cix::HOVER_RAY,
        true, QueryFilter::new().groups(group),
    ) {
        let hit = ray_dir * toi;
        let target = 9.81 * (hit + Vec2::new(0., Cix::HOVER_RAY)) + Vec2::new(0., -vel.linvel.y);

        **grounded = true;
        *force += ExternalForce::at_point(target, ray_pos, center);
    } else {
        **grounded = false;
    }
}

pub fn cix_update_head_sys(
    time: Res<Time>,
    mut cix: Query<&mut TextureAtlasSprite, With<Cix>>,
) {
    let absin = (time.elapsed_seconds() * Cix::WAVE_SCALE).sin() / 2. + 0.5;

    let mut sprite = cix.single_mut();
    sprite.color = Cix::COLOR.start().lerp(*Cix::COLOR.end(), absin);
    sprite.custom_size = Some(Vec2::splat((Cix::RADIUS.start() + absin * (Cix::RADIUS.end() - Cix::RADIUS.start())) * 2.));
}

pub fn cix_update_direction_sys(
    time: Res<Time>,
    mut cix: Query<&mut CixDirection>,
) {
    let mut dir = cix.single_mut();
    if dir.progress < 1. {
        dir.progress = (dir.progress + time.delta_seconds() * CixDirection::TURN_SPEED).min(1.);
    }
}
