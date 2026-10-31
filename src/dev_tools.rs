//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{dev_tools::states::log_transitions, prelude::*};
use leafwing_input_manager::common_conditions::action_just_pressed;

use crate::{GlobalAction, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(action_just_pressed(GlobalAction::ToggleDebug)),
    );
}

fn toggle_debug_ui(mut options: ResMut<GlobalUiDebugOptions>) {
    options.toggle();
}
