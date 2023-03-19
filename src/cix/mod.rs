use bevy::{
    prelude::*,
    sprite::Anchor,
};

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
    )).with_children(|builder| {
        builder.spawn((
            CixEye,
            SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: CIX_EYE_COLOR,
                    index: atlas.get(&atlases, &sprites.eye),
                    custom_size: Some(Vec2::splat(CIX_RADIUS.start() * 2.)),
                    ..default()
                },
                texture_atlas: atlas.clone_weak(),
                transform: Transform::from_xyz(0., 0., 2.),
                ..default()
            },
        ));

        for (i, &attire) in CIX_ATTIRE_ALL.into_iter().enumerate() {
            let anchor = attire.anchor();
            let offset = CIX_ATTIRE_SIZE * anchor;
            let layer = 4. - (i as f32 / CIX_ATTIRE_ALL.len() as f32);

            builder.spawn((
                attire,
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: atlas.get(&atlases, attire.sprite(&sprites)),
                        custom_size: Some(CIX_ATTIRE_SIZE),
                        anchor: Anchor::Custom(anchor),
                        ..default()
                    },
                    texture_atlas: atlas.clone_weak(),
                    transform: Transform::from_xyz(offset.x, offset.y - 36., layer),
                    ..default()
                },
            ));
        }
    });
}
