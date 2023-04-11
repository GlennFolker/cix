use bevy::prelude::*;

use crate::{
    CixSprites, GameAtlas,
    CixStates,
    Timed,
};

#[derive(Component)]
pub struct CixSpawn;
impl CixSpawn {
    pub const TIME: f64 = 1.2;
}

#[derive(Resource, Deref, DerefMut, Copy, Clone)]
pub struct CixSpawnPos(pub Vec2);

pub fn cix_init_spawn_sys(mut commands: Commands, pos: Res<CixSpawnPos>) {
    commands.spawn((
        (
            CixSpawn,
            Timed {
                life: 0.,
                lifetime: CixSpawn::TIME,
            },
        ),
        SpatialBundle::from(Transform::from_translation(pos.extend(50.))),
    ));
}

pub fn cix_update_spawn_sys(
    mut commands: Commands, mut state: ResMut<NextState<CixStates>>,
    spawn: Query<(&Timed, &GlobalTransform), With<CixSpawn>>,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<CixSprites>, atlas: Res<GameAtlas>,
) {
    let (&timed, &global_transform) = spawn.single();
    if timed.ended() {
        crate::cix_spawn(&mut commands, &atlases, &sprites, &atlas, global_transform);
        state.set(CixStates::Alive);
    }
}
