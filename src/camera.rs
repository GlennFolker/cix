use bevy::{
    prelude::*,
    core_pipeline::bloom::BloomSettings,
    math::DVec2,
    window::PrimaryWindow,
};

pub const CAMERA_VIEW: DVec2 = DVec2::new(1440., 900.);

#[derive(Resource, Deref, DerefMut, Copy, Clone)]
pub struct CameraPos(pub Vec2);

pub fn camera_spawn_sys(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        },
        BloomSettings {
            intensity: 0.4,
            ..BloomSettings::NATURAL
        },
    ));
}

pub fn camera_viewport_sys(
    camera_pos: Res<CameraPos>,
    mut camera: Query<(&Camera, &mut OrthographicProjection, &mut Transform)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    windows: Query<(Entity, &Window)>,
    images: Res<Assets<Image>>,
) {
    let (camera, mut proj, mut trns) = camera.single_mut();
    trns.translation.x = camera_pos.x;
    trns.translation.y = camera_pos.y;

    if
        let Some(target) = camera.target.normalize(primary_window.get_single().ok()) &&
        let Some(info) = target.get_render_target_info(&windows, &images)
    {
        let scl = info.scale_factor;
        let logical_width = info.physical_size.x as f64 / scl;
        let logical_height = info.physical_size.y as f64 / scl;
        proj.scale = (CAMERA_VIEW.x / logical_width).min(CAMERA_VIEW.y / logical_height) as f32;
    }
}
