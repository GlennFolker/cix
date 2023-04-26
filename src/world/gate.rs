use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    GROUP_GATE,
    EnvironmentSprites, GameAtlas,
    CixAction, CixActState,
    WorldStart, WorldObject,
    Timed,
};

#[derive(Component)]
pub struct Gate {
    pub level: String,
}

pub fn spawn_gate(
    commands: &mut Commands,
    atlases: &Assets<TextureAtlas>,
    sprites: &EnvironmentSprites, atlas: &GameAtlas,
    level: String, pos: Vec2,
) {
    commands.spawn((
        WorldObject,
        Gate { level },
        SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(atlas.index(atlases, &sprites.gate)),
            texture_atlas: atlas.clone_weak(),
            transform: Transform::from_translation(pos.extend(5.)),
            ..default()
        },
        (
            RigidBody::Fixed,
            Sensor,
            CollisionGroups::new(GROUP_GATE, Group::ALL),
            Collider::cuboid(32., 64.),
        ),
    ));
}

pub fn update_gate_sys(
    mut commands: Commands,
    context: Res<RapierContext>,
    cix: Query<(Entity, &CixActState)>,
    gates: Query<(Entity, &Gate)>,
    mut level: Query<&mut LevelSet>,
    objects: Query<Entity, With<WorldObject>>,
) {
    let Ok((cix, input)) = cix.get_single() else { return };
    let Some(axis) = input.axis_pair(CixAction::Move) else { return };
    if axis.y() <= 0. { return };

    let mut level = level.single_mut();
    for (e, gate) in &gates {
        if let Some(true) = context.intersection_pair(cix, e) {
            for object in &objects {
                commands.entity(object).despawn_recursive();
            }

            commands.spawn((
                WorldStart,
                Timed::new(WorldStart::FADE_DURATION),
            ));

            *level = LevelSet::from_iid(gate.level.clone());
            continue;
        }
    }
}
