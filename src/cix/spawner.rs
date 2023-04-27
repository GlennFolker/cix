use bevy::{
    prelude::*,
    sprite::Anchor,
};

use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{
    GROUP_CIX,
    GenericSprites, CixSprites, GameAtlas,
    Cix, CixGrounded, CixLastGrounded, CixHovered, CixDirection, CixAction, CixJumpState,
    CixEye, CixAttire, CixArm, CixArmTarget,
    CixAttack, CixAttackState, CixLaserChargeParticle,
    Health,
};

pub fn cix_spawn(
    commands: &mut Commands,
    atlases: &Assets<TextureAtlas>,
    generic_sprites: &GenericSprites, cix_sprites: &CixSprites, atlas: &GameAtlas,
    global_transform: GlobalTransform,
) {
    let group = CollisionGroups::new(GROUP_CIX, !GROUP_CIX);
    commands.spawn((
        (
            Cix,
            CixGrounded(false), CixLastGrounded(None), CixHovered(false),
            CixAttack::default(), CixAttackState::default(),
            CixDirection {
                right: true,
                progress: 1.,
            },
            Health::new(100.),
        ),
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                color: *Cix::COLOR.start(),
                index: atlas.index(atlases, &cix_sprites.head),
                custom_size: Some(Vec2::splat(Cix::RADIUS.start() * 2.)),
                ..default()
            },
            texture_atlas: atlas.clone_weak(),
            transform: global_transform.into(),
            global_transform,
            ..default()
        },
        (
            RigidBody::Dynamic,
            Collider::ball(*Cix::RADIUS.start()),
            group,
            ActiveEvents::COLLISION_EVENTS,
        ),
        (
            Velocity::default(),
            ExternalForce::default(),
            ExternalImpulse::default(),
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
        (
            CixJumpState::default(),
            InputManagerBundle {
                action_state: default(),
                input_map: InputMap::default()
                    .insert(VirtualDPad {
                        up: KeyCode::W.into(),
                        down: KeyCode::S.into(),
                        left: KeyCode::A.into(),
                        right: KeyCode::D.into(),
                    }, CixAction::Move)
                    .insert(KeyCode::Space, CixAction::Jump)
                    .insert(MouseButton::Left, CixAction::Attack)
                    .build(),
            },
        ),
    )).with_children(|builder| {
        for _ in 0..12 {
            builder.spawn((
                CixLaserChargeParticle::default(),
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: atlas.index(atlases, &generic_sprites.circle),
                        color: Color::NONE,
                        custom_size: Some(Vec2::splat(0.)),
                        ..default()
                    },
                    texture_atlas: atlas.clone_weak(),
                    ..default()
                },
            ));
        }

        builder.spawn((
            CixEye,
            SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: CixEye::COLOR,
                    index: atlas.index(atlases, &cix_sprites.eye),
                    ..default()
                },
                texture_atlas: atlas.clone_weak(),
                transform: Transform::from_xyz(0., 0., 2.),
                ..default()
            },
        ));

        for (i, &attire) in CixAttire::ALL.iter().enumerate() {
            let offset = attire.offset();
            let layer = 4. - (i as f32 / CixAttire::ALL.len() as f32);

            builder.spawn((
                attire,
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: atlas.index(atlases, attire.sprite(cix_sprites)),
                        ..default()
                    },
                    texture_atlas: atlas.clone_weak(),
                    transform: Transform::from_translation(Vec2::new(offset.x, offset.y - CixAttire::OFFSET).extend(layer)),
                    ..default()
                },
            ));
        }

        for (i, &arm) in CixArm::ALL.iter().enumerate() {
            let offset = arm.offset();
            let arm_len = arm.length();
            let layer = if i == 0 { 5. } else { -0.01 };

            let (anchor_upper, anchor_lower) = arm.anchor();
            let ((rect_upper, index_upper), (rect_lower, index_lower)) = {
                let (sprite_upper, sprite_lower) = arm.sprites(cix_sprites);
                (atlas.rect_index(atlases, sprite_upper), atlas.rect_index(atlases, sprite_lower))
            };

            builder.spawn((
                arm,
                CixArmTarget(None),
                SpatialBundle::from(Transform::from_xyz(offset.x, offset.y - CixAttire::OFFSET, layer)),
            )).with_children(|builder| {
                builder.spawn(SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: index_upper,
                        anchor: Anchor::Custom(Vec2::new(anchor_upper, 0.5) / rect_upper.size()),
                        ..default()
                    },
                    texture_atlas: atlas.clone_weak(),
                    ..default()
                });

                builder.spawn(SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: index_lower,
                        anchor: Anchor::Custom(Vec2::new(anchor_lower, 0.5) / rect_lower.size()),
                        ..default()
                    },
                    texture_atlas: atlas.clone_weak(),
                    transform: Transform::from_xyz(arm_len, 0., 0.),
                    ..default()
                });
            });
        }
    });
}
