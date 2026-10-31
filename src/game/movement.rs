//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the character within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

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
            max_speed: 400.0,
            acceleration: 400.0,
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
        let desired_velocity = controller.max_speed * controller.intent;
        let direction_change_factor = 1.5 - 0.5 * desired_velocity;
        let max_acceleration = direction_change_factor
            .normalize_or_zero()
            .dot(velocity.linear)
            * controller.acceleration;
        let walk_acceleration =
            (desired_velocity * time.delta_secs()).clamp_length_max(max_acceleration);

        external_force.force = walk_acceleration * mass_properties.get().mass;
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct CameraTarget;

pub fn camera_follow_system(
    time: Res<Time>,
    target_transform: Query<&Transform, With<CameraTarget>>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<CameraTarget>)>,
) {
    if let (Ok(target), Ok(mut camera_transform)) = (
        target_transform.single().map(|t| t.translation.xy()),
        camera_query.single_mut(),
    ) {
        let current = camera_transform.translation.xy();
        let lerped = Vec2::lerp(current, target, 1.0 - (-4.0 * time.delta_secs()).exp());
        camera_transform.translation = lerped.extend(camera_transform.translation.z);
    }
}
