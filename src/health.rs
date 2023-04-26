use bevy::prelude::*;

#[derive(Component, Copy, Clone)]
pub struct Health {
    pub amount: f32,
    pub max: f32,
}

impl Health {
    #[inline]
    pub fn new(max: f32) -> Self {
        Self {
            amount: max,
            max,
        }
    }

    #[inline]
    pub fn dead(self) -> bool {
        self.amount <= 0.
    }
}

#[derive(Copy, Clone)]
pub struct DeathEvent(pub Entity);

pub fn health_update_sys(mut writer: EventWriter<DeathEvent>, mut healths: Query<(Entity, &mut Health)>) {
    for (e, mut health) in &mut healths {
        if health.amount > health.max {
            health.amount = health.max;
        } else if health.dead() {
            writer.send(DeathEvent(e));
        }
    }
}

pub fn health_post_update_sys(mut commands: Commands, mut reader: EventReader<DeathEvent>) {
    for &DeathEvent(e) in &mut reader {
        commands.entity(e).despawn_recursive();
    }
}
