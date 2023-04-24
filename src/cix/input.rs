use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{
    ext::*,
    PIXELS_PER_METER,
    Cix, CixGrounded, CixLastGrounded, CixDirection,
    CixArmTarget,
};

pub const CIX_MOVE_VEL: f32 = 2.;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum CixAction {
    Move,
    Jump,
    Attack,
    Action,
}

impl CixAction {
    pub const ATTACK_DST: f32 = 56.;
}

pub type CixActState = ActionState<CixAction>;

#[derive(Component, Copy, Clone, Default)]
pub struct CixJumpState {
    pub jump_time: Option<f64>,
    pub last_jump_time: Option<f64>,
    pub buffer_time: Option<f64>,
}

impl CixJumpState {
    pub const IMPULSE: f32 = 0.5;
    pub const FORCE: f32 = 8.4;

    pub const DURATION: f64 = 0.4;
    pub const BUFFER_TIME: f64 = 0.14;
    pub const COYOTE_TIME: f64 = 0.1;
    pub const COYOTE_COOLDOWN: f64 = 0.3;
}

pub fn cix_move_sys(mut cix: Query<
    (&CixGrounded, &GlobalTransform, &ActionState<CixAction>, &Velocity, &mut ExternalForce),
    With<Cix>,
>) {
    let (&grounded, &global_trns, input, &vel, mut force) = cix.single_mut();
    let Some(axis) = input.axis_pair(CixAction::Move) else { return };

    let move_x = axis.x();
    let target_vel = CIX_MOVE_VEL * PIXELS_PER_METER * move_x;
    let center = global_trns.translation().truncate();

    if move_x != 0. || *grounded {
        let f = Vec2::new(target_vel - vel.linvel.x, 0.);
        *force += ExternalForce::at_point(f, center, center);
    }
}

pub fn cix_flip_direction_sys(
    window: Query<&Window>, camera: Query<(&Camera, &GlobalTransform)>,
    mut cix: Query<(&CixActState, &mut CixDirection, &GlobalTransform)>,
) {
    let Ok(window) = window.get_single() else { return };
    let (input, mut dir, &global_trns) = cix.single_mut();
    let (camera, &camera_trns) = camera.single();

    let Some(right) = (if input.pressed(CixAction::Attack) && let Some(pos) = window
        .cursor_position()
        .and_then(|pos| camera.viewport_to_world_2d(&camera_trns, pos))
    {
        Some(pos.x > global_trns.translation().x)
    } else if let Some(axis) = input.axis_pair(CixAction::Move) {
        let move_x = axis.x();
        if move_x != 0. {
            Some(move_x > 0.)
        } else {
            None
        }
    } else {
        None
    }) else { return };

    if dir.right != right {
        dir.right = right;
        dir.progress = 1. - dir.progress;
    }
}

pub fn cix_jump_sys(
    time: Res<Time>,
    mut cix: Query<(
        &mut CixJumpState, &CixActState,
        &GlobalTransform, &CixGrounded, &CixLastGrounded,
        &mut ExternalForce, &mut ExternalImpulse,
    ), With<Cix>>,
) {
    let (mut state, input, &global_trns, &grounded, &last_grounded, mut force, mut impulse) = cix.single_mut();
    if input.pressed(CixAction::Jump) {
        let current = time.elapsed_seconds_f64();
        if let Some(jump_time) = state.jump_time {
            let f = 1. - (current - jump_time).min(CixJumpState::DURATION) / CixJumpState::DURATION;
            let f = (f * f * f) as f32;

            let trns = global_trns.translation().truncate();
            *force += ExternalForce::at_point(Vec2::new(0., CixJumpState::FORCE * PIXELS_PER_METER * f), trns, trns);
        } else {
            let mut jump = false;
            if input.just_pressed(CixAction::Jump) {
                if *grounded {
                    jump = true;
                } else if
                    let Some(last_grounded) = *last_grounded && current - last_grounded <= CixJumpState::COYOTE_TIME &&
                    state.last_jump_time.map(|last_jump_time| current - last_jump_time > CixJumpState::COYOTE_COOLDOWN).unwrap_or(true)
                {
                    jump = true;
                } else {
                    state.buffer_time = Some(current);
                }
            }

            if !jump && *grounded && let Some(buffer_time) = state.buffer_time && current - buffer_time <= CixJumpState::BUFFER_TIME {
                jump = true;
            }

            if jump {
                state.jump_time = Some(current);
                state.last_jump_time = Some(current);

                let trns = global_trns.translation().truncate();
                *impulse += ExternalImpulse::at_point(Vec2::new(0., CixJumpState::IMPULSE * PIXELS_PER_METER), trns, trns);
            }
        }
    } else {
        state.jump_time = None;
        state.buffer_time = None;
    }
}

pub fn cix_attack_sys(
    window: Query<&Window>, camera: Query<(&Camera, &GlobalTransform)>,
    mut cix: Query<(&CixActState, &CixDirection), With<Cix>>, mut arms: Query<(&mut CixArmTarget, &GlobalTransform)>,
) {
    let Ok(window) = window.get_single() else { return };
    let (camera, &camera_trns) = camera.single();

    let (input, &dir) = cix.single_mut();
    if input.pressed(CixAction::Attack) && let Some(pos) = window
        .cursor_position()
        .and_then(|pos| camera.viewport_to_world_2d(&camera_trns, pos))
    {
        let mut prog = dir.progress;
        prog = prog * prog * (3. - 2. * prog);

        let p = if dir.right { prog } else { 1. - prog };
        for (mut arm, &arm_trns) in &mut arms {
            **arm = Some(Vec2::from_angle(
                (-(pos - arm_trns.translation().truncate()).angle_between(Vec2::X))
                .angle_clamp_range(
                    if p >= 0.5 { 0. } else { f32::PI },
                    (90f32 + (1. - (p * 2. - 1.).abs()) * 30f32).to_radians(),
                )
            ) * CixAction::ATTACK_DST);
        }
    } else {
        for (mut arm, _) in &mut arms {
            **arm = None;
        }
    }
}
