use bevy::{
    prelude::*,
    sprite::Anchor,
};

use bevy_rapier2d::prelude::*;

use crate::{
    CixSprites, GameAtlas,
};

mod attire;
mod head;
mod particle;
mod fire;
mod eye;

pub use attire::*;
pub use head::*;
pub use particle::*;
pub use fire::*;
pub use eye::*;

#[derive(Component)]
pub struct Cix;

pub fn cix_spawn_sys(
    mut commands: Commands,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<CixSprites>, atlas: Res<GameAtlas>,
) {
    commands.spawn((
        Cix,
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                color: *CIX_COLOR.start(),
                index: atlas.get(&atlases, &sprites.head),
                custom_size: Some(Vec2::splat(CIX_RADIUS.start() * 2.)),
                ..default()
            },
            texture_atlas: atlas.clone_weak(),
            transform: Transform::from_xyz(0., 0., 50.),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::ball(*CIX_RADIUS.start()),
        LockedAxes::ROTATION_LOCKED_Z,
    )).with_children(|builder| {
        builder.spawn((
            CixEye,
            SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: CIX_EYE_COLOR,
                    index: atlas.get(&atlases, &sprites.eye),
                    ..default()
                },
                texture_atlas: atlas.clone_weak(),
                transform: Transform::from_xyz(0., 0., 2.),
                ..default()
            },
        ));

        for (i, &attire) in CIX_ATTIRE_ALL.iter().enumerate() {
            let anchor = attire.anchor();
            let sprite = attire.sprite(&sprites);
            let layer = 4. - (i as f32 / CIX_ATTIRE_ALL.len() as f32);

            let (index, size) = {
                let atlas = atlases.get(&atlas).expect("Texture atlas deallocated");
                let index = atlas.get_texture_index(sprite).expect("Invalid texture atlas sprite");
                (index, atlas.textures[index].size())
            };

            let angle = attire.joint_angle();
            builder.spawn((
                attire,
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index,
                        anchor: Anchor::Custom(anchor / size),
                        ..default()
                    },
                    texture_atlas: atlas.clone_weak(),
                    transform: Transform::from_xyz(0., 0., layer),
                    ..default()
                },
                RigidBody::Dynamic,
                ImpulseJoint::new(builder.parent_entity(), RevoluteJointBuilder::new()
                    .limits([-angle / 2., angle / 2.])
                    .local_anchor1(Vec2::new(anchor.x, anchor.y - 54.))
                    .local_anchor2(Vec2::new(0., 0.))
                ),
            )).with_children(|builder| {
                let ((offset_x, offset_y), collider) = attire.collider();
                builder.spawn((
                    collider, Sensor,
                    ColliderMassProperties::Mass(0.1 / CIX_ATTIRE_ALL.len() as f32),
                    TransformBundle::from(Transform::from_xyz(offset_x - anchor.x, offset_y - anchor.y, 0.)),
                ));
            });
        }
    });
}
