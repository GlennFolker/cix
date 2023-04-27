use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    GROUP_CIX, GROUP_STATIC,
    EndStates,
    Health, Flower,
};

pub fn collide_sys(
    end_state_now: Res<State<EndStates>>,
    mut end_state: ResMut<NextState<EndStates>>,
    mut events: EventReader<CollisionEvent>,
    mut healths: Query<&mut Health>,
    groups: Query<&CollisionGroups>,
    flowers: Query<&Flower>,
) {
    for &event in &mut events {
        match event {
            CollisionEvent::Started(a, b, _) => {
                if let Some((mut health, group, other_group)) = {
                    if
                        let Ok(health) = healths.get_mut(a) &&
                        let Ok(&other_group) = groups.get(b)
                    {
                        Some((health, *groups.get(a).unwrap_or(&CollisionGroups::new(Group::ALL, Group::ALL)), other_group))
                    } else if
                        let Ok(health) = healths.get_mut(b) &&
                        let Ok(&other_group) = groups.get(a)
                    {
                        Some((health, *groups.get(b).unwrap_or(&CollisionGroups::new(Group::ALL, Group::ALL)), other_group))
                    } else {
                        None
                    }
                } &&
                    other_group.memberships.contains(GROUP_STATIC) &&
                    group.memberships.intersects(other_group.filters) &&
                    other_group.memberships.intersects(group.filters)
                {
                    health.amount = 0.;
                } else if
                    end_state_now.0 != EndStates::Yes &&
                    let Ok(&group) = groups.get(a) &&
                    let Ok(&other_group) = groups.get(b) && (
                        (group.memberships.contains(GROUP_CIX) && flowers.contains(b)) ||
                        (other_group.memberships.contains(GROUP_CIX) && flowers.contains(a))
                    )
                {
                    end_state.set(EndStates::Yes);
                }
            },
            _ => {},
        }
    }
}
