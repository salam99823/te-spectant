//! Player-specific behavior.

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    game::{
        animation::PlayerAnimation,
        collision_groups::{PLAYER_GROUP, WALL_GROUP},
        movement::{CameraTarget, MovementController},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();
    app.add_plugins((InputManagerPlugin::<PlayerAction>::default(),));

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        record_player_directional_input
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );
}

/// The player character.
pub fn player(
    player_assets: &PlayerAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 3, 4, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();

    (
        Player,
        Collider::ball(15.0),
        CollisionGroups::new(WALL_GROUP, PLAYER_GROUP),
        Sprite::from_atlas_image(
            player_assets.sprite.clone(),
            TextureAtlas {
                layout: texture_atlas_layout,
                index: player_animation.get_atlas_index(),
            },
        ),
        player_animation,
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
#[require(
    Name::new("Player"),
    InputMap<PlayerAction> = PlayerAction::input_map(),
    Transform::from_scale(Vec2::splat(4.0).extend(1.0)),
    CameraTarget,
    MovementController,
    RigidBody::Dynamic
)]
struct Player;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    #[actionlike(DualAxis)]
    Move,
    Interact,
}

impl PlayerAction {
    pub fn input_map() -> InputMap<PlayerAction> {
        InputMap::new([(PlayerAction::Interact, KeyCode::KeyE)])
            .with_dual_axis(PlayerAction::Move, VirtualDPad::wasd())
    }
}

fn record_player_directional_input(
    mut controller_query: Query<
        (&mut MovementController, &ActionState<PlayerAction>),
        With<Player>,
    >,
) {
    for (mut controller, action) in &mut controller_query {
        controller.intent = action
            .clamped_axis_pair(&PlayerAction::Move)
            .normalize_or_zero();
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    sprite: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            sprite: assets
                .load_builder()
                .with_settings(|settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                })
                .load("images/player.png"),
            steps: vec![],
        }
    }
}
