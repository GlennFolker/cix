use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::{
    prelude::*,
    helpers::square_grid::neighbors::Neighbors,
};
use bevy_rapier2d::prelude::*;

use crate::{
    ext::*,
    GROUP_STOP_PIERCE, GROUP_GROUND,
    StaticEnemySprites, GameAtlas,
    LdtkWorld,
    CameraPos, CixSpawnPos, CixStates,
    EnemyGears, EnemyGear,
    Timed,
};

mod fade;

pub use fade::*;

#[derive(Component)]
pub struct WorldStart;
impl WorldStart {
    pub const FADE_DURATION: f64 = 2.;
}

#[derive(Resource)]
pub struct WorldInit;

pub fn world_start_sys(mut commands: Commands, world: Res<LdtkWorld>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: world.clone_weak(),
        ..default()
    });

    commands.spawn((
        WorldStart,
        Timed::new(WorldStart::FADE_DURATION),
    ));
}

pub fn world_post_start_sys(
    mut commands: Commands,
    mut camera_pos: ResMut<CameraPos>, mut cix_pos: ResMut<CixSpawnPos>,
    mut gears: ResMut<EnemyGears>,
    added_entities: Query<(&EntityInstance, &GlobalTransform), Added<EntityInstance>>,
    added_tiles: Query<(Entity, &TilemapId, &TilePos, &IntGridCell), Added<IntGridCell>>,
    tiles: Query<&IntGridCell>,
    tilemaps: Query<(&LayerMetadata, &TileStorage)>,
    atlases: Res<Assets<TextureAtlas>>,
    enemy_sprites: Res<StaticEnemySprites>, atlas: Res<GameAtlas>,
    mut has_started: Local<bool>,
) {
    if *has_started { return };

    let mut started = false;
    for (inst, &trns) in &added_entities {
        if !started { started = true; }

        let pos = trns.translation().truncate();
        match inst.identifier.as_ref() {
            "cix" => {
                **camera_pos = pos;
                **cix_pos = pos;
            },
            "barrier" => {
                
            },
            "gear" => {
                let FieldValue::EntityRef(ref reference) = inst.field_instances.iter()
                    .find(|inst| &inst.identifier == "link").unwrap()
                    .value
                else { unreachable!() };
                let FieldValue::Float(Some(diameter)) = inst.field_instances.iter()
                    .find(|inst| &inst.identifier == "diameter").unwrap()
                    .value
                else { unreachable!() };
                let FieldValue::Color(color) = inst.field_instances.iter()
                    .find(|inst| &inst.identifier == "color").unwrap()
                    .value
                else { unreachable!() };

                gears.insert(inst.iid.clone(), commands.spawn((
                    EnemyGear {
                        radius: diameter / 2.,
                        link: None,
                        link_iid: reference.as_ref().map(|r| r.entity_iid.clone())
                    },
                    (
                        RigidBody::Fixed,
                        CollisionGroups::new(GROUP_STOP_PIERCE, Group::ALL),
                        Collider::ball(diameter * 16.),
                    ),
                    SpriteSheetBundle {
                        sprite: TextureAtlasSprite {
                            index: atlas.index(&atlases, &enemy_sprites.gear),
                            custom_size: Some(Vec2::splat(diameter * 32.)),
                            color,
                            ..default()
                        },
                        texture_atlas: atlas.clone_weak(),
                        transform: Transform::from_translation(pos.extend(10.)),
                        ..default()
                    },
                )).id());
            },
            _ => {},
        }
    }

    for (e, &tilemap_id, &pos, &cell) in &added_tiles {
        if !started { started = true; }

        const NONE: i32 = 0;
        const GND: i32 = 1;
        const GND_SLOPE: i32 = 2;
        const GND_LSLOPE: i32 = 3;

        let is = |tile: Option<Entity>, expect: i32| if expect != NONE {
            tile
                .and_then(|tile| tiles.get(tile).ok())
                .map(|cell| cell.value == expect)
                .unwrap_or(false)
        } else {
            tile.is_none()
        };

        let (layer, storage) = tilemaps.get(tilemap_id.0).unwrap();
        let s = layer.grid_size as f32 / 2.;
        let n = Neighbors::get_square_neighboring_positions(&pos, &storage.size, true).entities(storage);
        let group = CollisionGroups::new(GROUP_STOP_PIERCE | GROUP_GROUND, !GROUP_GROUND);

        'select: {
            if cell.value == GND {
                let (lslope_tl, lslope_tr, lslope_bl, lslope_br) = (
                    is(n.east, GND) && is(n.south_east, GND_LSLOPE), is(n.west, GND) && is(n.south_west, GND_LSLOPE),
                    is(n.west, GND) && is(n.south, GND_LSLOPE), is(n.east, GND) && is(n.south, GND_LSLOPE),
                );

                if lslope_tl || lslope_tr || lslope_bl || lslope_br {
                    let x = if lslope_tl || lslope_bl { s } else { -s };
                    let y = if lslope_tl || lslope_tr { s } else { 0. };

                    commands.entity(e).insert((
                        RigidBody::Fixed,
                        group,
                        Collider::trimesh(
                            vec![Vec2::new(-x, y), Vec2::new(x, y - s), Vec2::new(x, y - s * 2.), Vec2::new(-x, y - s)],
                            vec![[0, 1, 2], [2, 3, 0]],
                        ),
                    ));

                    break 'select;
                }

                let (slope_l, slope_r) = (
                    is(n.south, GND_SLOPE) && is(n.south_east, GND),
                    is(n.south, GND_SLOPE) && is(n.south_west, GND)
                );

                if slope_l || slope_r {
                    let x = if slope_l { s } else { -s };
                    commands.entity(e).insert((
                        RigidBody::Fixed,
                        group,
                        Collider::trimesh(
                            vec![Vec2::new(-x, s), Vec2::new(x, -s), Vec2::new(x, -s * 2.), Vec2::new(-x, 0.)],
                            vec![[0, 1, 2], [2, 3, 0]],
                        ),
                    ));

                    break 'select;
                }

                let (edge_l, edge_r) = (
                    is(n.west, GND) && is(n.east, NONE),
                    is(n.east, GND) && is(n.west, NONE),
                );

                if edge_l || edge_r {
                    let x = if edge_l { s } else { -s };
                    commands.entity(e).insert((
                        RigidBody::Fixed,
                        group,
                        Collider::triangle(Vec2::new(-x, s), Vec2::new(x, s), Vec2::new(-x, 0.)),
                    ));

                    break 'select;
                }

                commands.entity(e).insert((
                    RigidBody::Fixed,
                    group,
                    Collider::trimesh(
                        vec![Vec2::new(-s, s), Vec2::new(s, s), Vec2::new(s, 0.), Vec2::new(-s, 0.)],
                        vec![[0, 1, 2], [2, 3, 0]],
                    ),
                ));
            }
        }

        if started {
            *has_started = true;
            commands.insert_resource(WorldInit);
        }
    }
}

pub fn world_start_update_sys(
    init: Option<Res<WorldInit>>,
    mut start: Query<&mut Timed, With<WorldStart>>,
    mut fade: Query<&mut TextureAtlasSprite, With<WorldFade>>,
    mut state: ResMut<NextState<CixStates>>,
) {
    let mut timed = start.single_mut();
    if init.is_none() {
        timed.life = 0.;
        return;
    }

    let mut fade = fade.single_mut();

    let mut f = timed.fin();
    f = f * f * (3. - 2. * f);
    fade.color = Color::BLACK.lerp(Color::NONE, f);

    if timed.ended() {
        state.set(CixStates::Spawning);
    }
}
