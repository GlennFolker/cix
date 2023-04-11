use bevy::{
    prelude::*,
    window::PrimaryWindow,
};

use crate::{
    CameraPos,
    GenericSprites,
    GameAtlas,
};

#[derive(Component)]
pub struct WorldFade;

pub fn world_fade_add_sys(
    mut commands: Commands,
    camera: Query<(&Camera, &OrthographicProjection, &GlobalTransform)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    windows: Query<(Entity, &Window)>,
    images: Res<Assets<Image>>,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<GenericSprites>, atlas: Res<GameAtlas>,
) {
    let (camera, proj, &camera_trns) = camera.single();
    let camera_trns = camera_trns.translation();

    commands.spawn((
        WorldFade,
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                color: Color::BLACK,
                index: atlas.index(&atlases, &sprites.square),
                custom_size: camera.target
                    .normalize(primary_window.get_single().ok())
                    .and_then(|target| target.get_render_target_info(&windows, &images))
                    .map(|info| {
                        println!("{info:?}");
                        let scl = info.scale_factor;
                        Vec2::new(
                            (info.physical_size.x as f64 / scl) as f32,
                            (info.physical_size.y as f64 / scl) as f32,
                        ) * proj.scale * 2.
                    }),
                ..default()
            },
            texture_atlas: atlas.clone_weak(),
            transform: Transform::from_xyz(camera_trns.x, camera_trns.y, camera_trns.z - 0.1),
            ..default()
        },
    ));
}

pub fn world_fade_update_sys(
    camera_pos: Res<CameraPos>,
    camera: Query<(&Camera, &OrthographicProjection, &GlobalTransform)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    windows: Query<(Entity, &Window)>,
    images: Res<Assets<Image>>,
    mut fade: Query<(&mut Transform, &mut TextureAtlasSprite), With<WorldFade>>,
) {
    let (camera, proj, &camera_trns) = camera.single();
    let (mut trns, mut sprite) = fade.single_mut();

    if
        let Some(target) = camera.target.normalize(primary_window.get_single().ok()) &&
        let Some(info) = target.get_render_target_info(&windows, &images)
    {
        let scl = info.scale_factor;
        let logical_width = (info.physical_size.x as f64 / scl) as f32;
        let logical_height = (info.physical_size.y as f64 / scl) as f32;

        trns.translation = camera_pos.extend(camera_trns.translation().z - 0.1);
        sprite.custom_size = Some(Vec2::new(logical_width, logical_height) * proj.scale * 2.);
    }
}
