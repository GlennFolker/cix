use bevy::prelude::*;

#[derive(Component, Copy, Clone, Default)]
pub struct Timed {
    pub life: f64,
    pub lifetime: f64,
}

impl Timed {
    #[inline]
    pub fn fin(self) -> f32 {
        self.fin_64() as f32
    }

    #[inline]
    pub fn fin_64(self) -> f64 {
        (self.life / self.lifetime).clamp(0., 1.)
    }
}

pub fn timed_update_sys(
    mut commands: Commands, time: Res<Time>,
    mut all: Query<(Entity, &mut Timed)>,
) {
    let delta = time.delta_seconds_f64();
    for (entity, mut timed) in &mut all {
        timed.life += delta;
        if timed.life >= timed.lifetime {
            commands.entity(entity).despawn_recursive();
        }
    }
}
