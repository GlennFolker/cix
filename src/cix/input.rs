use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{
    PIXELS_PER_METER,
    Cix, CixGrounded, CixDirection,
};

pub const CIX_MOVE_VEL: f32 = 2.;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum CixAction {
    Move,
    Jump,
}

pub type CixActState = ActionState<CixAction>;

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

pub fn cix_flip_direction_sys(mut cix: Query<(&CixActState, &mut CixDirection), Changed<CixActState>>) {
    let (input, mut dir) = cix.single_mut();
    let Some(axis) = input.axis_pair(CixAction::Move) else { return };

    let move_x = axis.x();
    if move_x != 0. {
        let right = move_x > 0.;
        if dir.right != right {
            dir.right = right;
            dir.progress = 1. - dir.progress;
        }
    }
}
