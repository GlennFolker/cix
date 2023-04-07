use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
	ext::*,
	CixStates,
	Timed,
};

mod fade;

pub use fade::*;

#[derive(Component)]
pub struct WorldStart;
impl WorldStart {
	pub const FADE_DURATION: f64 = 2.;
}

pub fn world_start_sys(mut commands: Commands) {
    commands.spawn((
        RigidBody::Fixed,
        //Collider::cuboid(1000., 8.),
        Collider::heightfield(vec![0., 3., 2., 4., 5., 1., 3., 1., 2., 0.], Vec2::new(800., 10.)),
        TransformBundle::from(Transform::from_xyz(0., -160., 0.)),
    ));

    commands.spawn((
    	WorldStart,
    	Timed {
    		life: 0.,
    		lifetime: WorldStart::FADE_DURATION,
    	},
    ));
}

pub fn world_start_update_sys(
	start: Query<&Timed, With<WorldStart>>,
	mut fade: Query<&mut TextureAtlasSprite, With<WorldFade>>,
	mut state: ResMut<NextState<CixStates>>,
) {
	let &timed = start.single();
	let mut fade = fade.single_mut();
	fade.color = Color::BLACK.lerp(Color::NONE, timed.fin());

	if timed.ended() {
		state.set(CixStates::Spawning);
	}
}
