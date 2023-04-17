use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::{
    prelude::*,
    helpers::square_grid::neighbors::Neighbors,
};
use bevy_rapier2d::prelude::*;

use crate::{
    ext::*,
    GROUP_GND,
    LdtkWorld,
    CameraPos, CixSpawnPos, CixStates,
    Timed,
};

mod fade;

pub use fade::*;

#[derive(Component)]
pub struct WorldStart;
impl WorldStart {
    pub const FADE_DURATION: f64 = 2.;
}

pub fn world_start_sys(mut commands: Commands, world: Res<LdtkWorld>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: world.clone_weak(),
        ..default()
    });

    commands.spawn((
        WorldStart,
        Timed {
            life: 0.,
            lifetime: WorldStart::FADE_DURATION,
        },
    ));
}

pub fn world_post_start_sys(
    mut commands: Commands,
    mut camera_pos: ResMut<CameraPos>, mut cix_pos: ResMut<CixSpawnPos>,
    added_entities: Query<(&EntityInstance, &GlobalTransform), Added<EntityInstance>>,
    added_tiles: Query<(Entity, &TilemapId, &TilePos, &IntGridCell), Added<IntGridCell>>,
    tiles: Query<&IntGridCell>,
    tilemaps: Query<(&LayerMetadata, &TileStorage)>,
) {
    for (inst, &trns) in &added_entities {
        if &inst.identifier == "cix" {
            let pos = trns.translation().truncate();
            **camera_pos = pos;
            **cix_pos = pos;
        }
    }

    for (e, &tilemap_id, &pos, &cell) in &added_tiles {
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
        let group = CollisionGroups::new(GROUP_GND, !GROUP_GND);

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
    }
}

pub fn world_start_update_sys(
    start: Query<&Timed, With<WorldStart>>,
    mut fade: Query<&mut TextureAtlasSprite, With<WorldFade>>,
    mut state: ResMut<NextState<CixStates>>,
) {
    let &timed = start.single();
    let mut fade = fade.single_mut();

    let mut f = timed.fin();
    f = f * f * (3. - 2. * f);
    fade.color = Color::BLACK.lerp(Color::NONE, f);

    if timed.ended() {
        state.set(CixStates::Spawning);
    }
}
