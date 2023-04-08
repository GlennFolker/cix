use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::ext::*;
use std::ops::RangeInclusive as RangeIncl;

mod arm;
mod attire;
mod input;
mod particle;
mod fire;
mod eye;
mod spawn;

pub use arm::*;
pub use attire::*;
pub use input::*;
pub use particle::*;
pub use fire::*;
pub use eye::*;
pub use spawn::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum CixStates {
    #[default]
    Nonexistent,
    Spawning,
    Alive,
    Dead,
}

#[derive(Component)]
pub struct Cix;
impl Cix {
    pub const COLOR: RangeIncl<Color> = Color::rgba(0.2, 1.4, 2.5, 0.3)..=Color::rgba(0.4, 1.8, 3., 0.36);
    pub const WAVE_SCALE: f32 = 16.;
    pub const RADIUS: RangeIncl<f32> = 24f32..=26f32;
    pub const HOVER_RAY: f32 = 100.;
}

#[derive(Component, Deref, DerefMut, Copy, Clone)]
pub struct CixGrounded(pub bool);

#[derive(Component, Copy, Clone)]
pub struct CixDirection {
    pub right: bool,
    pub progress: f32,
}

impl CixDirection {
    pub const TURN_SPEED: f32 = 3.2;
}

pub fn cix_pre_update_sys(mut cix: Query<&mut ExternalForce, With<Cix>>) {
    let mut force = cix.single_mut();
    *force = default();
}

pub fn cix_update_sys(
    context: Res<RapierContext>,
    mut cix: Query<(&mut CixGrounded, &Collider, &GlobalTransform, &CollisionGroups, &Velocity, &mut ExternalForce), With<Cix>>,
) {
    let (mut grounded, collider, &global_trns, &group, &vel, mut force) = cix.single_mut();

    let ray_pos = global_trns.translation().truncate();
    let ray_dir = -Vec2::Y;

    if let Some((_, toi)) = context.cast_shape(
        ray_pos, 0., ray_dir,
        collider, Cix::HOVER_RAY, QueryFilter::new().groups(group),
    ) {
        let hit = ray_dir * toi.toi;
        let target = 9.81 * (hit + Vec2::new(0., Cix::HOVER_RAY)) + Vec2::new(0., -vel.linvel.y);

        **grounded = true;
        *force += ExternalForce::at_point(target, ray_pos, ray_pos);
    } else {
        **grounded = false;
    }
}

pub fn cix_update_head_sys(
    time: Res<Time>,
    mut cix: Query<&mut TextureAtlasSprite, With<Cix>>,
) {
    let absin = (time.elapsed_seconds() * Cix::WAVE_SCALE).sin() / 2. + 0.5;

    let mut sprite = cix.single_mut();
    sprite.color = Cix::COLOR.start().lerp(*Cix::COLOR.end(), absin);
    sprite.custom_size = Some(Vec2::splat((Cix::RADIUS.start() + absin * (Cix::RADIUS.end() - Cix::RADIUS.start())) * 2.));
}

pub fn cix_update_direction_sys(
    time: Res<Time>,
    mut cix: Query<&mut CixDirection>,
) {
    let mut dir = cix.single_mut();
    if dir.progress < 1. {
        dir.progress = (dir.progress + time.delta_seconds() * CixDirection::TURN_SPEED).min(1.);
    }
}
