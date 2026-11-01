use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    game::{
        collision_groups::{PLAYER, SENSOR},
        player::Player,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        handle_room_transitions
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Component, Debug, Clone, Copy, Reflect, FromTemplate)]
#[reflect(Component)]
#[require(
    Sensor,
    ActiveEvents::COLLISION_EVENTS,
    CollisionGroups::new(SENSOR, PLAYER),
    SolverGroups::new(SENSOR, Group::NONE)
)]
pub struct TransitionTrigger(#[entities] pub Entity);

fn handle_room_transitions(
    mut collision_events: MessageReader<CollisionEvent>,
    sensor_query: Query<&TransitionTrigger>,
    transform_query: Query<&Transform, Without<Player>>,
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            let trigger_data = if let Ok(trigger) = sensor_query.get(*entity1) {
                Some((trigger, *entity2))
            } else if let Ok(trigger) = sensor_query.get(*entity2) {
                Some((trigger, *entity1))
            } else {
                None
            };

            if let Some((&TransitionTrigger(target_entity), potential_player)) = trigger_data
                && let Ok((mut player_transform, mut player_velocity)) =
                    player_query.get_mut(potential_player)
                && let Ok(target) = transform_query.get(target_entity)
            {
                player_transform.translation =
                    target.translation.with_z(player_transform.translation.z);

                player_velocity.linear = Vec2::ZERO;
            }
        }
    }
}
