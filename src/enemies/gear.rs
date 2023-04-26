use bevy::{
    prelude::*,
    sprite::Anchor,
    utils::HashMap,
};
use bevy_rapier2d::prelude::*;

use crate::{
    ext::*,
    GROUP_STATIC, GROUP_STOP_PIERCE,
    GenericSprites, StaticEnemySprites, GameAtlas,
    WorldObject,
};

#[derive(Resource, Default, Deref, DerefMut)]
pub struct EnemyGears(pub HashMap<String, Entity>);

#[derive(Component)]
pub struct EnemyGear {
    pub radius: f32,
    pub link: Option<Entity>,
    pub link_iid: Option<String>,
}

impl EnemyGear {
    pub const ROTATE_SPEED: f32 = 4.;
}

pub fn spawn_enemy_gear(
    commands: &mut Commands,
    diameter: f32, color: Color,
    reference: Option<String>,
    pos: Vec2,
    atlases: &Assets<TextureAtlas>,
    enemy_sprites: &StaticEnemySprites, atlas: &GameAtlas,
) -> Entity {
    commands.spawn((
        WorldObject,
        EnemyGear {
            radius: diameter / 2.,
            link: None,
            link_iid: reference,
        },
        (
            RigidBody::Fixed,
            CollisionGroups::new(GROUP_STATIC | GROUP_STOP_PIERCE, Group::ALL),
            Collider::ball(diameter * 16.),
        ),
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: atlas.index(atlases, &enemy_sprites.gear),
                custom_size: Some(Vec2::splat(diameter * 32.)),
                color,
                ..default()
            },
            texture_atlas: atlas.clone_weak(),
            transform: Transform::from_translation(pos.extend(20.)),
            ..default()
        },
    )).id()
}

pub fn enemy_gear_init_sys(
    mut commands: Commands,
    gears: Res<EnemyGears>,
    mut set: ParamSet<(
        Query<(Entity, &mut EnemyGear, &TextureAtlasSprite), Added<EnemyGear>>,
        Query<(&EnemyGear, &TextureAtlasSprite, &GlobalTransform)>,
    )>,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<GenericSprites>, atlas: Res<GameAtlas>,
) {
    let mut links = Vec::new();
    for (e, mut gear, sprite) in &mut set.p0() {
        commands.entity(e).with_children(|builder| { builder.spawn(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: atlas.index(&atlases, &sprites.circle),
                color: sprite.color,
                custom_size: Some(Vec2::splat(gear.radius * 16.)),
                ..default()
            },
            texture_atlas: atlas.clone_weak(),
            ..default()
        }); });

        if let Some(ref iid) = gear.link_iid {
            gear.link = gears.get(iid).copied();
            if let Some(link) = gear.link {
                links.push((e, link));
            }
        }
    }

    for (e, link) in links {
        let queue = set.p1();
        let [(from, from_sprite, &from_trns), (to, to_sprite, &to_trns)] = queue.many([e, link]);
        let from_trns = from_trns.translation();
        let to_trns = to_trns.translation().truncate();

        let angle = Vec2::X.angle_between(to_trns - from_trns.truncate());
        let mut positions = vec![Vec2::splat(0.); 4];

        for (index, i) in [-1., 1.].into_iter().enumerate() {
            let off = Vec2::from_angle(angle + f32::PI / 2. * i);
            let start = from_trns.truncate() + off * (from.radius * 8. - 2.25);
            let end = to_trns + off * (to.radius * 8. - 2.25);

            commands.spawn((
                WorldObject,
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: atlas.index(&atlases, &sprites.square),
                        color: from_sprite.color.lerp(to_sprite.color, 0.5).with_a(0.5),
                        anchor: Anchor::Custom(Vec2::new(-0.5, 0.)),
                        custom_size: Some(Vec2::new((end - start).length(), 4.5)),
                        ..default()
                    },
                    texture_atlas: atlas.clone_weak(),
                    transform: Transform::from_translation(start.extend(from_trns.z))
                        .with_rotation(Quat::from_axis_angle(Vec3::Z, Vec2::X.angle_between(end - start))),
                    ..default()
                },
            ));

            let origin = from_trns.truncate();
            if index == 0 {
                positions[0] = start - origin;
                positions[1] = end - origin;
            } else if index == 1 {
                positions[2] = end - origin;
                positions[3] = start - origin;
            }
        }

        commands.spawn((
            WorldObject,
            RigidBody::Fixed,
            CollisionGroups::new(GROUP_STATIC | GROUP_STOP_PIERCE, Group::ALL),
            Collider::polyline(positions, Some(vec![[0, 1], [1, 2], [2, 3], [3, 0]])),
            TransformBundle::from(Transform::from_translation(from_trns)),
        ));
    }
}

pub fn enemy_gear_update_sys(
    time: Res<Time>,
    mut gears: Query<(&EnemyGear, &mut Transform), With<EnemyGear>>,
) {
    let delta = time.delta_seconds() * 60.;
    for (gear, mut trns) in &mut gears {
        trns.rotation *= Quat::from_axis_angle(Vec3::Z, (delta * EnemyGear::ROTATE_SPEED / gear.radius).to_radians());
    }
}
