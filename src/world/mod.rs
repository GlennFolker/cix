use bevy::{
    prelude::*,
    utils::{
        HashSet, HashMap,
    },
};
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
    EnemyGears,
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
    added_tiles: Query<(Entity, &TilemapId, &TilePos, &IntGridCell, &GlobalTransform), Added<IntGridCell>>,
    tiles: Query<&IntGridCell>,
    tilemaps: Query<(&LayerMetadata, &TileStorage)>,
    atlases: Res<Assets<TextureAtlas>>,
    enemy_sprites: Res<StaticEnemySprites>, atlas: Res<GameAtlas>,
    start: Query<(), Added<WorldStart>>,
    mut has_started: Local<bool>,
) {
    if start.get_single().is_ok() {
        *has_started = false;
        println!("Starting...");
    }

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
                let FieldValue::Float(Some(height)) = inst.field_instances.iter()
                    .find(|inst| &inst.identifier == "height").unwrap()
                    .value
                else { unreachable!() };
                let FieldValue::Color(color) = inst.field_instances.iter()
                    .find(|inst| &inst.identifier == "color").unwrap()
                    .value
                else { unreachable!() };

                crate::spawn_enemy_barrier(
                    &mut commands,
                    height, color,
                    pos,
                    &atlases,
                    &enemy_sprites, &atlas,
                );
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

                gears.insert(inst.iid.clone(), crate::spawn_enemy_gear(
                    &mut commands,
                    diameter, color,
                    reference.as_ref().map(|r| r.entity_iid.clone()),
                    pos,
                    &atlases,
                    &enemy_sprites, &atlas,
                ));
            },
            _ => {},
        }
    }

    let mut flat = HashMap::default();

    let group = CollisionGroups::new(GROUP_STOP_PIERCE | GROUP_GROUND, !GROUP_GROUND);
    for (e, &tilemap_id, &pos, &cell, &tile_trns) in &added_tiles {
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
                        Collider::polyline(
                            vec![Vec2::new(-x, y), Vec2::new(x, y - s), Vec2::new(x, y - s * 2.), Vec2::new(-x, y - s)],
                            Some(vec![[0, 1], [1, 2], [2, 3], [3, 0]]),
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
                        Collider::polyline(
                            vec![Vec2::new(-x, s), Vec2::new(x, -s), Vec2::new(x, -s * 2.), Vec2::new(-x, 0.)], 
                            Some(vec![[0, 1], [1, 2], [2, 3], [3, 0]]),
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

                flat.insert(pos, tile_trns.translation().truncate());
            }
        }
    }

    let mut iterated = HashSet::default();
    let mut flatmost = Vec::new();
    for &pos in flat.keys() {
        if !iterated.insert(pos) { continue };

        let mut leftmost = pos;
        while leftmost.x > 0 && flat.contains_key(&TilePos { x: leftmost.x - 1, y: leftmost.y, }) {
            iterated.insert(TilePos { x: leftmost.x - 1, y: leftmost.y, });
            leftmost = TilePos { x: leftmost.x - 1, y: leftmost.y, };
        }

        let mut rightmost = pos;
        while flat.contains_key(&TilePos { x: rightmost.x + 1, y: rightmost.y, }) {
            iterated.insert(TilePos { x: rightmost.x + 1, y: rightmost.y, });
            rightmost = TilePos { x: rightmost.x + 1, y: rightmost.y, };
        }

        flatmost.push((leftmost, rightmost));
    }

    for (left, right) in flatmost {
        let left_pos = flat[&left];
        let right_pos = flat[&right];
        let center = (left_pos + right_pos) / 2. + Vec2::new(0., 8.);
        let len = right_pos.x - left_pos.x + 32.;

        commands.spawn((
            RigidBody::Fixed,
            group,
            Collider::cuboid(len / 2., 8.),
            TransformBundle::from(Transform::from_translation(center.extend(0.))),
        ));
    }

    if started {
        *has_started = true;
        commands.insert_resource(WorldInit);
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
