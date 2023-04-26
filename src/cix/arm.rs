use bevy::prelude::*;
use smallvec::SmallVec;

use crate::{
    ext::*,
    CixSprites, GameAtlas,
    CixDirection,
    CixAttire,
};

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum CixArm {
    Front,
    Back,
}

#[derive(Component, Deref, DerefMut, Copy, Clone)]
pub struct CixArmTarget(pub Option<Vec2>);

impl CixArm {
    pub const ALL: &'static [Self] = &[
        Self::Front,
        Self::Back,
    ];

    pub const TURN_SPEED_ACTIVE: f32 = 0.67;
    pub const TURN_SPEED_PASSIVE: f32 = 0.24;

    pub const TARGET_POINT: Vec2 = Vec2::new(1.5, -26.);

    #[inline]
    pub fn sprites(self, sprites: &CixSprites) -> (&Handle<Image>, &Handle<Image>) {
        use CixArm::*;
        match self {
            Front => (&sprites.arm_front_upper, &sprites.arm_front_lower),
            Back => (&sprites.arm_back_upper, &sprites.arm_back_lower),
        }
    }

    #[inline]
    pub fn offset(self) -> Vec2 {
        use CixArm::*;
        match self {
            Front => Vec2::new(-3., 2.),
            Back => Vec2::new(6., 0.),
        }
    }

    #[inline]
    pub fn anchor(self) -> (f32, f32) {
        use CixArm::*;
        match self {
            Front => (-17., -17.),
            Back => (-15., -15.),
        }
    }

    #[inline]
    pub fn length(self) -> f32 {
        use CixArm::*;
        match self {
            Front => 32.,
            Back => 30.,
        }
    }
}

pub fn cix_update_arm_sys(
    time: Res<Time>,
    cix: Query<&CixDirection>,
    mut arm: Query<(&CixArm, &Children, &CixArmTarget, &mut Transform)>,
    mut arms: Query<(&mut Transform, &mut TextureAtlasSprite), Without<CixArm>>,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<CixSprites>, atlas: Res<GameAtlas>,
) {
    let Ok(&dir) = cix.get_single() else { return };
    let mut prog = dir.progress;
    prog = prog * prog * (3. - 2. * prog);

    let size_prog = 1. - (0.5 - (prog - 0.5).abs()) * 2. * CixAttire::ROTATE_SHRINK;
    let anchor_prog = (prog * 2. - 1.) * if dir.right { 1. } else { -1. };
    let p = if dir.right { prog } else { 1. - prog };
    let flip = p < 0.5;
    let sign = if flip { -1. } else { 1. };

    for (&arm, segments, &target, mut arm_trns) in &mut arm {
        let offset = arm.offset();
        let arm_len = arm.length();

        arm_trns.translation.x = offset.x * anchor_prog;
        let (target_joint, end, speed) = if let Some(target) = *target {
            let mut mat1 = [Vec2::default(), Vec2::default()];
            let mut mat2 = [Vec2::default(), Vec2::default()];

            let mut attractor = target.rotate(Vec2::from_angle(-1f32.to_radians() * sign));
            attractor *= (((arm_len + arm_len) * (arm_len + arm_len)) / attractor.length_squared()).sqrt();
            attractor += target / 2.;

            mat2[0] = target.normalize();
            mat2[1] = (attractor - mat2[0] * attractor.dot(mat2[0])).normalize();
            mat1[0] = Vec2::new(mat2[0].x, mat2[1].x);
            mat1[1] = Vec2::new(mat2[0].y, mat2[1].y);

            let dist = (Vec2::new(mat2[0].dot(target), mat2[1].dot(target)).length() / 2.).clamp(0., arm_len);
            let src = Vec2::new(dist, (arm_len * arm_len - dist * dist).sqrt());
            (Vec2::new(mat1[0].dot(src), mat1[1].dot(src)), target, CixArm::TURN_SPEED_ACTIVE)
        } else {
            let joint = Vec2::from_angle((-90f32 - 10f32 * sign).to_radians()) * arm_len;
            (joint, Vec2::new(joint.x, joint.y - arm_len), CixArm::TURN_SPEED_PASSIVE)
        };

        let [(mut upper_trns, mut upper_sprite), (mut lower_trns, mut lower_sprite)] = arms.many_mut({
            let arms = segments.iter().copied().collect::<SmallVec<[Entity; 2]>>();
            arms.into_inner().unwrap()
        });

        let delta = speed * (time.delta_seconds() * 60.).min(1.);
        let cur_joint = lower_trns.translation.truncate();
        let angle_diff = Vec2::X
            .angle_between(cur_joint)
            .angle_dist_avoid(Vec2::X.angle_between(target_joint), if p >= 0.5 { f32::PI } else { 0. });
        let joint = cur_joint.rotate(Vec2::from_angle(angle_diff * delta));

        let upper_angle = Vec2::X.angle_between(joint);
        let lower_angle = {
            let from = lower_trns.rotation.to_axis_angle().1;
            let to = Vec2::X.angle_between(end - target_joint);
            let dist = from.angle_dist_avoid(to, upper_angle + std::f32::consts::PI) * delta;

            (from + dist).angle_wrap()
        };

        upper_trns.rotation = Quat::from_axis_angle(Vec3::Z, upper_angle);
        lower_trns.translation = joint.extend(0.);
        lower_trns.rotation = Quat::from_axis_angle(Vec3::Z, lower_angle);

        let (upper_handle, lower_handle) = arm.sprites(&sprites);
        for (handle, sprite) in [(upper_handle, &mut upper_sprite), (lower_handle, &mut lower_sprite)] {
            let sprite_size = atlas.rect(&atlases, handle).size();
            sprite.custom_size = Some(Vec2::new(
                sprite_size.x,
                sprite_size.y * size_prog,
            ));
            sprite.flip_y = flip;
        }/**/
    }
}
