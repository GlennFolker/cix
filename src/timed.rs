use bevy::prelude::*;

#[derive(Component, Copy, Clone)]
pub struct Timed {
    pub life: f64,
    pub lifetime: f64,
    pub backwards: bool,
    pub scale: f64,
}

impl Default for Timed {
    #[inline]
    fn default() -> Self {
        Self {
            life: 0.,
            lifetime: 0.,
            backwards: false,
            scale: 1.,
        }
    }
}

impl Timed {
    #[inline]
    pub fn new(lifetime: f64) -> Self {
        Self {
            lifetime,
            ..default()
        }
    }

    #[inline]
    pub fn fin(self) -> f32 {
        self.fin_64() as f32
    }

    #[inline]
    pub fn fin_64(self) -> f64 {
        (self.life / self.lifetime).clamp(0., 1.)
    }

    #[inline]
    pub fn ended(self) -> bool {
        self.life >= self.lifetime
    }
}

pub fn timed_update_sys(time: Res<Time>, mut all: Query<&mut Timed>) {
    let delta = time.delta_seconds_f64();
    for mut timed in &mut all {
        let d = delta * timed.scale;
        if timed.backwards {
            timed.life -= d;
        } else {
            timed.life += d;
        }
    }
}

pub fn timed_post_update_sys(mut commands: Commands, all: Query<(Entity, &Timed)>) {
    for (entity, &timed) in &all {
        if timed.ended() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
