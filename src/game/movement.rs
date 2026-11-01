use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{AppSystems, MainCamera, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (apply_movement, camera_follow_system)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(
    Velocity,
    ExternalForce,
    LockedAxes::ROTATION_LOCKED_Z,
    Damping {
        linear_damping: 4.0,
        angular_damping: 0.0
    }
)]
pub struct MovementController {
    /// The direction the character wants to move in.
    pub intent: Vec2,

    /// Maximum speed.
    pub max_speed: f32,

    /// Maximum acceleration.
    pub acceleration: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            intent: Vec2::ZERO,
            max_speed: 300.0,
            acceleration: 2000.0,
        }
    }
}

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(
        &MovementController,
        &Velocity,
        &ReadMassProperties,
        &mut ExternalForce,
    )>,
) {
    for (controller, velocity, mass_properties, mut external_force) in &mut movement_query {
        let desired_velocity = controller.intent * controller.max_speed;
        let velocity_diff = desired_velocity - velocity.linear;
        if velocity_diff.length_squared() < 500. {
            external_force.force = Vec2::ZERO;
            continue;
        }
        let acceleration =
            (velocity_diff / time.delta_secs()).clamp_length_max(controller.acceleration);
        external_force.force = acceleration * mass_properties.get().mass;
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct CameraTarget;

pub fn camera_follow_system(
    time: Res<Time>,
    target_transform: Query<&GlobalTransform, With<CameraTarget>>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<CameraTarget>)>,
) {
    if let (Ok(target), Ok(mut camera_transform)) = (
        target_transform.single().map(|t| t.translation().xy()),
        camera_query.single_mut(),
    ) {
        let current = camera_transform.translation.xy();
        if current.distance(target) > 10.0 {
            let lerped = Vec2::lerp(current, target, 1.0 - (-4.0 * time.delta_secs()).exp());
            camera_transform.translation = lerped.extend(camera_transform.translation.z);
        }
    }
}
