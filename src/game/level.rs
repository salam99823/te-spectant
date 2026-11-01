//! src/game/level.rs
use crate::game::{
    player::Player,
    scenes::{MansionConfig, MansionRonLoader, RoomConfig, RoomRonLoader, Wall},
    transition::TransitionTrigger,
};
use crate::screens::Screen;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Resource)]
pub struct CurrentMansionLoad {
    pub mansion_handle: Handle<MansionConfig>,
    pub room_handles: Vec<Handle<RoomConfig>>,
    pub spawned: bool,
}

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<MansionConfig>()
        .init_asset::<RoomConfig>()
        .register_asset_loader(MansionRonLoader)
        .register_asset_loader(RoomRonLoader);

    app.add_systems(
        Update,
        (monitor_mansion_async_load, resolve_trigger_relationships),
    );
}

pub fn spawn_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mansion_handle = asset_server.load::<MansionConfig>("scenes/mansion.ron");

    commands.insert_resource(CurrentMansionLoad {
        mansion_handle,
        room_handles: Vec::new(),
        spawned: false,
    });
}

fn monitor_mansion_async_load(
    mut commands: Commands,
    load_state: Option<ResMut<CurrentMansionLoad>>,
    mansion_assets: Res<Assets<MansionConfig>>,
    room_assets: Res<Assets<RoomConfig>>,
    asset_server: Res<AssetServer>,
) {
    let Some(mut state) = load_state else { return };
    if state.spawned {
        return;
    };

    if state.room_handles.is_empty() {
        if let Some(mansion) = mansion_assets.get(&state.mansion_handle) {
            for room_path in &mansion.rooms {
                state
                    .room_handles
                    .push(asset_server.load::<RoomConfig>(room_path));
            }
        }
        return;
    }

    let all_rooms_ready = state.room_handles.iter().all(|h| asset_server.is_loaded(h));
    if !all_rooms_ready {
        return;
    };

    commands
        .spawn((
            Name::new("Level"),
            Transform::from_scale(Vec2::splat(4.0).extend(1.0)),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
        ))
        .with_children(|commands| {
            for room_handle in &state.room_handles {
                let config = room_assets.get(room_handle).unwrap();
                let img = asset_server.load(&config.image_path);

                commands
                    .spawn((
                        Name::new(config.name.clone()),
                        RigidBody::Fixed,
                        Sprite::from_image(img),
                        Transform::from_translation(config.translation.extend(0.0)),
                    ))
                    .with_children(|commands| {
                        if !config.walls.is_empty() {
                            commands
                                .spawn((
                                    Name::new("Walls"),
                                    Transform::default(),
                                    Visibility::default(),
                                ))
                                .with_children(|commands| {
                                    for wall in &config.walls {
                                        commands.spawn((
                                            Wall,
                                            Collider::cuboid(
                                                wall.half_extents.x,
                                                wall.half_extents.y,
                                            ),
                                            Transform::from_translation(wall.position.extend(0.0)),
                                        ));
                                    }
                                });
                        }

                        if !config.triggers.is_empty() {
                            commands
                                .spawn((
                                    Name::new("Sensors"),
                                    Transform::default(),
                                    Visibility::default(),
                                ))
                                .with_children(|commands| {
                                    for trigger in &config.triggers {
                                        commands.spawn((
                                            Name::new(trigger.name.clone()),
                                            TransitionTrigger(Entity::PLACEHOLDER),
                                            UnresolvedTargetMarker(
                                                trigger.target_room_name.clone(),
                                            ),
                                            Collider::cuboid(
                                                trigger.half_extents.x,
                                                trigger.half_extents.y,
                                            ),
                                            Transform::from_translation(
                                                trigger.position.extend(0.0),
                                            ),
                                        ));
                                    }
                                });
                        }
                    });
            }
            commands.spawn(Player);
        });

    state.spawned = true;
}

#[derive(Component)]
struct UnresolvedTargetMarker(String);

fn resolve_trigger_relationships(
    mut commands: Commands,
    unresolved_query: Query<(Entity, &UnresolvedTargetMarker, &Name)>,
    rooms_query: Query<(Entity, &Name), Without<UnresolvedTargetMarker>>,
) {
    for (trigger_entity, unresolved, _) in &unresolved_query {
        for (room_entity, room_name) in &rooms_query {
            if room_name.as_str() == unresolved.0.as_str() {
                commands
                    .entity(trigger_entity)
                    .insert(TransitionTrigger(room_entity))
                    .remove::<UnresolvedTargetMarker>();
                break;
            }
        }
    }
}
