//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use bevy::prelude::*;
use rand::prelude::*;
use std::{cmp::Ordering, time::Duration};

use crate::{
    AppSystems, PausableSystems,
    audio::sound_effect,
    game::{movement::MovementController, player::PlayerAssets},
};

pub(super) fn plugin(app: &mut App) {
    // Animate and play sound effects based on controls.
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSystems::TickTimers),
            (
                update_animation_movement,
                update_animation_atlas,
                trigger_step_sound_effect,
            )
                .chain()
                .in_set(AppSystems::Update),
        )
            .in_set(PausableSystems),
    );
}

/// Update the animation timer.
fn update_animation_timer(time: Res<Time>, mut query: Query<&mut PlayerAnimation>) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

/// Update the sprite direction and animation state (idling/walking).
fn update_animation_movement(mut player_query: Query<(&MovementController, &mut PlayerAnimation)>) {
    for (controller, mut animation) in &mut player_query {
        let animation_state = if controller.intent == Vec2::ZERO {
            PlayerAnimationState::Idling
        } else {
            PlayerAnimationState::Walking(controller.intent)
        };
        animation.update_state(animation_state);
    }
}

/// Update the texture atlas to reflect changes in the animation.
fn update_animation_atlas(mut query: Query<(&mut PlayerAnimation, &mut Sprite)>) {
    for (mut animation, mut sprite) in &mut query {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        if animation.changed() {
            atlas.index = animation.get_atlas_index();
            animation.changed = false;
        }
    }
}

/// If the player is moving, play a step sound effect synchronized with the
/// animation.
fn trigger_step_sound_effect(
    mut commands: Commands,
    player_assets: If<Res<PlayerAssets>>,
    mut step_query: Query<&PlayerAnimation>,
) {
    if player_assets.steps.is_empty() {
        return;
    }
    for animation in &mut step_query {
        if let PlayerAnimationState::Walking(_) = animation.state
            && animation.changed()
            && (animation.frame == 2 || animation.frame == 5)
        {
            let rng = &mut rand::rng();
            let random_step = player_assets.steps.choose(rng).unwrap().clone();
            commands.spawn(sound_effect(random_step));
        }
    }
}

/// Component that tracks player's animation state.
/// It is tightly bound to the texture atlas we use.
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct PlayerAnimation {
    timer: Timer,
    frame: usize,
    state: PlayerAnimationState,
    changed: bool,
}

#[derive(Reflect, PartialEq, Default, Clone)]
pub enum PlayerAnimationState {
    #[default]
    Idling,
    Walking(Vec2),
}

impl PlayerAnimation {
    /// The number of idle frames.
    const IDLE_FRAMES: usize = 1;
    /// The duration of each idle frame.
    const IDLE_INTERVAL: Duration = Duration::from_secs(1);
    /// The number of walking frames.
    const WALKING_FRAMES: usize = 3;
    /// The duration of each walking frame.
    const WALKING_INTERVAL: Duration = Duration::from_millis(200);

    fn idling() -> Self {
        Self {
            timer: Timer::new(Self::IDLE_INTERVAL, TimerMode::Repeating),
            state: PlayerAnimationState::Idling,
            ..default()
        }
    }

    fn walking(dir: Vec2) -> Self {
        Self {
            timer: Timer::new(Self::WALKING_INTERVAL, TimerMode::Repeating),
            state: PlayerAnimationState::Walking(dir),
            ..default()
        }
    }

    /// Update animation timers.
    pub fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if !self.timer.is_finished() {
            return;
        }
        self.frame = (self.frame + 1)
            % match self.state {
                PlayerAnimationState::Idling => Self::IDLE_FRAMES,
                PlayerAnimationState::Walking(_) => Self::WALKING_FRAMES,
            };
    }

    /// Update animation state if it changes.
    pub fn update_state(&mut self, state: PlayerAnimationState) {
        if self.state != state {
            match state {
                PlayerAnimationState::Idling => *self = Self::idling(),
                PlayerAnimationState::Walking(dir) => *self = Self::walking(dir),
            }
        }
        self.changed = true;
    }

    /// Whether animation changed this tick.
    pub fn changed(&self) -> bool {
        self.changed || self.timer.is_finished()
    }

    /// Return sprite index in the atlas.
    pub fn get_atlas_index(&self) -> usize {
        match self.state {
            PlayerAnimationState::Idling => self.frame,
            PlayerAnimationState::Walking(dir) => {
                let start = match (dir.x.total_cmp(&0.0), dir.y.total_cmp(&0.0)) {
                    (_, Ordering::Less) => 0,
                    (_, Ordering::Greater) => 9,
                    (Ordering::Less, _) => 3,
                    (Ordering::Greater, _) => 6,
                    _ => 0,
                };
                start + self.frame
            }
        }
    }
}
