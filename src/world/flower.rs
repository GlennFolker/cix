use bevy::{
    prelude::*,
    sprite::Anchor,
};
use bevy_rapier2d::prelude::*;

use crate::{
    ext::*,
    GROUP_STOP_PIERCE,
    EnvironmentSprites, GenericSprites, GameAtlas,
    WorldObject,
};

#[derive(Component)]
pub struct Flower;

pub fn spawn_flower(
    commands: &mut Commands,
    atlases: &Assets<TextureAtlas>,
    env_sprites: &EnvironmentSprites, gen_sprites: &GenericSprites, atlas: &GameAtlas,
    pos: Vec2,
) {
    commands.spawn((
        WorldObject,
        Flower,
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: atlas.index(atlases, &gen_sprites.circle),
                color: Color::rgba(0.98, 1., 0.52, 1.),
                custom_size: Some(Vec2::splat(32.)),
                ..default()
            },
            texture_atlas: atlas.clone_weak(),
            transform: Transform::from_translation(pos.extend(40.)),
            ..default()
        },
        (
            RigidBody::Fixed,
            Sensor,
            CollisionGroups::new(GROUP_STOP_PIERCE, Group::ALL),
            Collider::ball(24.),
        ),
    )).with_children(|builder| {
        for i in 0..5 {
            let angle = (360. / 5. * i as f32).to_radians();
            builder.spawn(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: atlas.index(atlases, &env_sprites.petal),
                    anchor: Anchor::Custom(Vec2::new(0., -0.5)),
                    ..default()
                },
                texture_atlas: atlas.clone_weak(),
                transform: Transform::from_translation((Vec2::from_angle(angle) * 12.).extend(-1.))
                    .with_rotation(Quat::from_axis_angle(Vec3::Z, angle - f32::PI / 2.)),
                ..default()
            });
        }
    });
}

pub fn update_flower_sys(
    time: Res<Time>,
    flower: Query<&Children, With<Flower>>,
    mut petals: Query<&mut TextureAtlasSprite>,
) {
    let Ok(flower) = flower.get_single() else { return };
    for (i, &petal) in flower.iter().enumerate() {
        let mut sprite = petals.get_mut(petal).unwrap();
        let mut sin = ((time.elapsed_seconds() + f32::PI * 2. / 10. * i as f32).sin() + 1.) / 2.;
        sin *= sin * sin;
        sin = sin / 5. + 0.8;

        sprite.custom_size = Some(Vec2::splat(sin * 32.));
    }
}
