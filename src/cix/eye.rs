use bevy::prelude::*;

pub const CIX_EYE_COLOR: Color = Color::rgb(1., 2.4, 4.8);
pub const CIX_EYE_DEVIATE: f32 = 8.;
pub const CIX_EYE_TILT: f32 = 0.2;
pub const CIX_EYE_FOCUS: f32 = 320.;

#[derive(Component)]
pub struct CixEye;

pub fn cix_update_eye_sys(
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut eye: Query<&mut Transform, With<CixEye>>,
) {
    let window = window.single();
    let (camera, camera_trns) = camera.single();
    let mut trns = eye.single_mut();

    if let Some(pos) = window
        .physical_cursor_position()
        .and_then(|pos| camera.viewport_to_world_2d(camera_trns, pos))
    {
        let vec = (pos - Vec2::new(trns.translation.x, trns.translation.y)).clamp_length_max(CIX_EYE_FOCUS);
        let mut len = vec.length() / CIX_EYE_FOCUS;
        len = 1. - (len - 1.) * (len - 1.);
        len *= CIX_EYE_DEVIATE;

        let deviate = vec * ((len * len) / vec.length_squared()).sqrt();

        let tilt = vec / CIX_EYE_FOCUS;
        let mut tilt_left =
            tilt.dot(Vec2::from_angle(45f32.to_radians())).max(0.) +
            tilt.dot(Vec2::from_angle(225f32.to_radians())).max(0.);
        tilt_left = (1. - (tilt_left - 1.) * (tilt_left - 1.)) * CIX_EYE_TILT;

        let mut tilt_right =
            tilt.dot(Vec2::from_angle(135f32.to_radians())).max(0.) +
            tilt.dot(Vec2::from_angle(315f32.to_radians())).max(0.);
        tilt_right = (1. - (tilt_right - 1.) * (tilt_right - 1.)) * -CIX_EYE_TILT;

        trns.translation.x = deviate.x;
        trns.translation.y = deviate.y;
        trns.rotation = Quat::from_axis_angle(Vec3::Z, tilt_left + tilt_right);
    }
}
