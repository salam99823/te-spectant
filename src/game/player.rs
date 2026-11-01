//! Player-specific behavior.

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    game::{
        animation::PlayerAnimation,
        collision_groups::{ENEMY, PLAYER, SENSOR, WALL},
        movement::{CameraTarget, MovementController},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();
    app.add_plugins((InputManagerPlugin::<PlayerAction>::default(),));

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        record_player_input
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );
}

fn add_sprite(mut world: DeferredWorld, hooks: HookContext) {
    let entity = hooks.entity;

    let player_assets = world.resource::<PlayerAssets>().clone();
    let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 3, 4, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    world
        .commands()
        .entity(entity)
        .insert(Sprite::from_atlas_image(
            player_assets.sprite.clone(),
            TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            },
        ));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect, FromTemplate)]
#[reflect(Component)]
#[require(
    Name::new("Player"),
    InputMap<PlayerAction> = PlayerAction::input_map(),
    Transform,
    PlayerAnimation,
    CameraTarget,
    MovementController,
    RigidBody::Dynamic,
    CollisionGroups::new(PLAYER, WALL | ENEMY | SENSOR),
    Collider::capsule_y(3.0, 9.0),
)]
#[component(on_add = add_sprite)]
pub struct Player;

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

fn record_player_input(
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
            sprite: assets.load("images/player.png"),
            steps: vec![],
        }
    }
}
