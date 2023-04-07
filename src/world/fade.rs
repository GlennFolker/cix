use bevy::{
    prelude::*,
    window::PrimaryWindow,
};

use crate::{
    GenericSprites,
    GameAtlas,
};

#[derive(Component)]
pub struct WorldFade;

pub fn world_fade_add_sys(
    mut commands: Commands,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<GenericSprites>, atlas: Res<GameAtlas>,
) {
    commands.spawn((
        WorldFade,
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                color: Color::BLACK,
                index: atlas.index(&atlases, &sprites.square),
                ..default()
            },
            texture_atlas: atlas.clone_weak(),
            ..default()
        },
    ));
}

pub fn world_fade_update_sys(
    camera: Query<(&Camera, &GlobalTransform)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    windows: Query<(Entity, &Window)>,
    images: Res<Assets<Image>>,
    mut fade: Query<(&mut Transform, &mut TextureAtlasSprite), With<WorldFade>>,
) {
    let (camera, &camera_trns) = camera.single();
    let (mut trns, mut sprite) = fade.single_mut();

    if
        let Some(target) = camera.target.normalize(primary_window.iter().next()) &&
        let Some(info) = target.get_render_target_info(&windows, &images)
    {
        let scl = info.scale_factor;
        let logical_width = (info.physical_size.x as f64 / scl) as f32;
        let logical_height = (info.physical_size.y as f64 / scl) as f32;

        let camera_trns = camera_trns.translation();
        trns.translation = Vec3::new(camera_trns.x, camera_trns.y, camera_trns.z - 0.01);
        sprite.custom_size = Some(Vec2::new(logical_width * 2., logical_height * 2.));
    }
}
