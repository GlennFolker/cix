use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    GROUP_STATIC,
    Health,
};

pub fn collide_sys(
    mut events: EventReader<CollisionEvent>,
    mut healths: Query<&mut Health>,
    groups: Query<&CollisionGroups>,
) {
    for &event in &mut events {
        match event {
            CollisionEvent::Started(a, b, _) => {
                let (mut health, group, other_group) = if
                    let Ok(health) = healths.get_mut(a) &&
                    let Ok(&other_group) = groups.get(b)
                {
                    (health, *groups.get(a).unwrap_or(&CollisionGroups::new(Group::ALL, Group::ALL)), other_group)
                } else if
                    let Ok(health) = healths.get_mut(b) &&
                    let Ok(&other_group) = groups.get(a)
                {
                    (health, *groups.get(b).unwrap_or(&CollisionGroups::new(Group::ALL, Group::ALL)), other_group)
                } else { continue };

                if
                    other_group.memberships.contains(GROUP_STATIC) &&
                    group.memberships.intersects(other_group.filters) &&
                    other_group.memberships.intersects(group.filters)
                {
                    health.amount = 0.;
                }
            },
            _ => {},
        }
    }
}
