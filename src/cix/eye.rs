use bevy::prelude::*;

#[derive(Component)]
pub struct CixEye;
impl CixEye {
    pub const COLOR: Color = Color::rgb(1., 2.4, 4.8);
    pub const DEVIATE: f32 = 8.;
    pub const TILT: f32 = 0.2;
    pub const FOCUS: f32 = 480.;
}

pub fn cix_update_eye_sys(
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut eye: Query<(&mut Transform, &GlobalTransform), With<CixEye>>,
) {
    let Ok(window) = window.get_single() else { return };
    let (camera, camera_trns) = camera.single();
    let (mut trns, &global_trns) = eye.single_mut();

    if let Some(pos) = window
        .physical_cursor_position()
        .and_then(|pos| camera.viewport_to_world_2d(camera_trns, pos))
    {
        let eye_trns = global_trns.translation();

        let vec = (pos - Vec2::new(eye_trns.x, eye_trns.y)).clamp_length_max(CixEye::FOCUS);
        let mut len = vec.length() / CixEye::FOCUS;
        len = 1. - (len - 1.) * (len - 1.);
        len *= CixEye::DEVIATE;

        let deviate = vec * ((len * len) / vec.length_squared()).sqrt();

        let tilt = vec / CixEye::FOCUS;
        let mut tilt_left =
            tilt.dot(Vec2::from_angle(45f32.to_radians())).max(0.) +
            tilt.dot(Vec2::from_angle(225f32.to_radians())).max(0.);
        tilt_left = (1. - (tilt_left - 1.) * (tilt_left - 1.)) * CixEye::TILT;

        let mut tilt_right =
            tilt.dot(Vec2::from_angle(135f32.to_radians())).max(0.) +
            tilt.dot(Vec2::from_angle(315f32.to_radians())).max(0.);
        tilt_right = (1. - (tilt_right - 1.) * (tilt_right - 1.)) * -CixEye::TILT;

        trns.translation.x = deviate.x;
        trns.translation.y = deviate.y;
        trns.rotation = Quat::from_axis_angle(Vec3::Z, tilt_left + tilt_right);
    }
}
