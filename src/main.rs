// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod asset_tracking;
mod audio;
#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod menus;
mod screens;
mod theme;

use bevy::{asset::AssetMetaCheck, camera::visibility::RenderLayers, prelude::*};
use bevy_rapier2d::plugin::{
    BroadPhaseOptimizationStrategy, IntegrationParameters, NoUserData, PhysicsSet,
    RapierBevyComponentApply, RapierConfiguration, RapierContextInitialization,
    RapierPhysicsPlugin, RapierTransformPropagateSet, TimestepMode,
};
use leafwing_input_manager::prelude::*;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Te Spectant".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
            InputManagerPlugin::<GlobalAction>::default(),
            RapierPhysicsPlugin::<NoUserData>::default()
                .with_custom_initialization({
                    let length_unit = 100.0;
                    let integration_parameters = IntegrationParameters {
                        length_unit,
                        ..default()
                    };

                    RapierContextInitialization::InitializeDefaultRapierContext {
                        integration_parameters,
                        rapier_configuration: RapierConfiguration {
                            gravity: Vec2::ZERO,
                            ..RapierConfiguration::new(length_unit)
                        },
                        broad_phase_optimization_strategy: BroadPhaseOptimizationStrategy::default(
                        ),
                    }
                })
                .with_default_system_setup(false),
        ));

        // Add other plugins.
        app.add_plugins((
            asset_tracking::plugin,
            audio::plugin,
            game::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            menus::plugin,
            screens::plugin,
            theme::plugin,
        ));

        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));

        // Set up the physics system sets.
        {
            app.configure_sets(
                PostUpdate,
                (
                    PhysicsSet::SyncBackend,
                    PhysicsSet::StepSimulation,
                    PhysicsSet::Writeback,
                )
                    .chain()
                    .before(TransformSystems::Propagate)
                    .run_if(in_state(Pause(false))),
            );
            app.configure_sets(
                PostUpdate,
                (RapierTransformPropagateSet, RapierBevyComponentApply)
                    .in_set(PhysicsSet::SyncBackend),
            );
            app.add_systems(
                PostUpdate,
                (
                    RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::SyncBackend),
                    RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::StepSimulation)
                        .in_set(PhysicsSet::StepSimulation),
                    RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::Writeback)
                        .in_set(PhysicsSet::Writeback),
                ),
            );
            app.init_resource::<TimestepMode>();
        }

        // Setup
        app.add_systems(Startup, (spawn_camera, spawn_action_manager));
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;

#[derive(Component)]
pub struct MainCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Name::new("Camera"), Camera2d, MainCamera));
    commands.spawn((
        Name::new("UI Camera"),
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        IsDefaultUiCamera,
        RenderLayers::layer(1),
    ));
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum GlobalAction {
    Pause,
    Escape,
    ToggleDebug,
}

impl GlobalAction {
    fn input_map() -> InputMap<Self> {
        InputMap::default()
            .with(Self::Pause, KeyCode::Pause)
            .with(Self::Pause, KeyCode::KeyP)
            .with(Self::Escape, KeyCode::Escape)
            .with(Self::ToggleDebug, KeyCode::Backquote)
    }
}

fn spawn_action_manager(mut commands: Commands) {
    commands.spawn(GlobalAction::input_map());
}
