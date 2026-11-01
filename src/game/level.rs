use crate::PausableSystems;
use crate::screens::Screen;
use crate::{
    AppSystems,
    game::{
        player::Player,
        scenes::{MansionConfig, MansionRonLoader, RoomConfig, RoomRonLoader, Wall},
        transition::TransitionTrigger,
    },
};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Resource)]
pub struct CurrentMansionLoad {
    pub mansion_handle: Handle<MansionConfig>,
    pub room_handles: Vec<Handle<RoomConfig>>,
    pub spawned_root: Option<Entity>,
}

#[derive(Component, Clone)]
struct RoomIdMarker(AssetId<RoomConfig>);

#[derive(Component)]
struct UnresolvedTargetMarker(String);

#[derive(Component)]
struct WallContainerMarker;

#[derive(Component)]
struct SensorContainerMarker;

#[derive(Component)]
struct UpdateRoomGeometryMarker;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<MansionConfig>()
        .init_asset::<RoomConfig>()
        .register_asset_loader(MansionRonLoader)
        .register_asset_loader(RoomRonLoader);

    app.add_systems(
        Update,
        (
            monitor_mansion_async_load,
            resolve_trigger_relationships,
            handle_hot_reload,
        )
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
    app.add_systems(OnEnter(Screen::Gameplay), spawn_level);
}

pub fn spawn_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mansion_handle = asset_server.load::<MansionConfig>("scenes/mansion.ron");

    let root = commands
        .spawn((
            Name::new("Level"),
            Transform::from_scale(Vec2::splat(4.0).extend(1.0)),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
        ))
        .id();
    commands.insert_resource(CurrentMansionLoad {
        mansion_handle,
        room_handles: Vec::new(),
        spawned_root: Some(root),
    });
}

fn monitor_mansion_async_load(
    mut commands: Commands,
    load_state: Option<ResMut<CurrentMansionLoad>>,
    mansion_assets: Res<Assets<MansionConfig>>,
    room_assets: Res<Assets<RoomConfig>>,
    asset_server: Res<AssetServer>,
    existing_rooms: Query<(Entity, &RoomIdMarker), Without<UpdateRoomGeometryMarker>>,
    update_rooms: Query<(Entity, &RoomIdMarker), With<UpdateRoomGeometryMarker>>,
    wall_containers: Query<(Entity, &ChildOf), With<WallContainerMarker>>,
    sensor_containers: Query<(Entity, &ChildOf), With<SensorContainerMarker>>,
    player_query: Query<Entity, With<Player>>,
    children_query: Query<&Children>,
    wall_query: Query<&Wall>,
    sensor_marker_query: Query<&Name, With<TransitionTrigger>>,
) {
    let Some(mut state) = load_state else { return };

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
    }

    let Some(level_root) = state.spawned_root else {
        return;
    };

    for room_handle in &state.room_handles {
        let room_id = room_handle.id();

        let is_updating = update_rooms.iter().find(|(_, marker)| marker.0 == room_id);
        let already_exists = existing_rooms.iter().any(|(_, marker)| marker.0 == room_id);

        if already_exists && is_updating.is_none() {
            continue;
        }

        let config = room_assets.get(room_handle).unwrap();

        if let Some((room_entity, _)) = is_updating {
            commands
                .entity(room_entity)
                .remove::<UpdateRoomGeometryMarker>();

            for (w_container, w_parent) in &wall_containers {
                if w_parent.parent() == room_entity {
                    let mut current_walls = Vec::new();
                    if let Ok(children) = children_query.get(w_container) {
                        for child in children.iter() {
                            if wall_query.get(child).is_ok() {
                                current_walls.push(child);
                            }
                        }
                    }

                    let current_len = current_walls.len();
                    let config_len = config.walls.len();

                    if current_len > config_len {
                        for &wall in &current_walls[config_len..current_len] {
                            commands.entity(wall).despawn();
                        }
                    }
                    for (config, wall) in config.walls.iter().zip(current_walls) {
                        commands
                            .entity(wall)
                            .insert_if_neq(Transform::from_translation(config.position.extend(0.0)))
                            .insert(Collider::cuboid(
                                config.half_extents.x,
                                config.half_extents.y,
                            ));
                    }
                }

                for (s_container, s_parent) in &sensor_containers {
                    if s_parent.parent() == room_entity {
                        let mut current_sensors = Vec::new();
                        if let Ok(children) = children_query.get(s_container) {
                            for child in children.iter() {
                                if sensor_marker_query.get(child).is_ok() {
                                    current_sensors.push(child);
                                }
                            }
                        }

                        let current_len = current_sensors.len();
                        let config_len = config.triggers.len();

                        if current_len > config_len {
                            for &sensor in &current_sensors[config_len..current_len] {
                                commands.entity(sensor).despawn();
                            }
                        }
                        for (config, sensor) in config.triggers.iter().zip(current_sensors) {
                            commands
                                .entity(sensor)
                                .insert_if_neq(Transform::from_translation(
                                    config.position.extend(0.0),
                                ))
                                .insert((
                                    UnresolvedTargetMarker(config.target_room_name.clone()),
                                    Collider::cuboid(config.half_extents.x, config.half_extents.y),
                                ));
                        }
                    }
                }
            }
        } else {
            let img = asset_server.load(&config.image_path);
            commands.entity(level_root).with_children(|commands| {
                commands
                    .spawn((
                        Name::new(config.name.clone()),
                        RoomIdMarker(room_id),
                        RigidBody::Fixed,
                        Sprite::from_image(img),
                        Transform::from_translation(config.translation.extend(0.0)),
                    ))
                    .with_children(|commands| {
                        commands
                            .spawn((
                                Name::new("Walls"),
                                WallContainerMarker,
                                Transform::default(),
                                Visibility::default(),
                            ))
                            .with_children(|commands| {
                                for wall in &config.walls {
                                    commands.spawn((
                                        Wall,
                                        Collider::cuboid(wall.half_extents.x, wall.half_extents.y),
                                        Transform::from_translation(wall.position.extend(0.0)),
                                    ));
                                }
                            });

                        commands
                            .spawn((
                                Name::new("Sensors"),
                                SensorContainerMarker,
                                Transform::default(),
                                Visibility::default(),
                            ))
                            .with_children(|commands| {
                                for trigger in &config.triggers {
                                    commands.spawn((
                                        Name::new(trigger.name.clone()),
                                        TransitionTrigger(Entity::PLACEHOLDER),
                                        UnresolvedTargetMarker(trigger.target_room_name.clone()),
                                        Collider::cuboid(
                                            trigger.half_extents.x,
                                            trigger.half_extents.y,
                                        ),
                                        Transform::from_translation(trigger.position.extend(0.0)),
                                    ));
                                }
                            });
                    });
            });
        }
    }

    if player_query.is_empty() {
        commands.entity(level_root).with_children(|commands| {
            commands.spawn(Player);
        });
    }
}

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

fn handle_hot_reload(
    mut commands: Commands,
    load_state: Option<ResMut<CurrentMansionLoad>>,
    mut mansion_events: MessageReader<AssetEvent<MansionConfig>>,
    mut room_events: MessageReader<AssetEvent<RoomConfig>>,
    rooms_query: Query<(Entity, &RoomIdMarker)>,
    mut transition_triggers: Query<(Entity, &mut TransitionTrigger)>,
    room_assets: Res<Assets<RoomConfig>>,
) {
    let Some(mut state) = load_state else { return };

    for event in mansion_events.read() {
        let AssetEvent::Modified { id } = event else {
            continue;
        };
        if *id == state.mansion_handle.id() {
            state.room_handles.clear();
            if let Some(root_entity) = state.spawned_root {
                commands.entity(root_entity).despawn();
            }
            state.spawned_root = None;
            return;
        }
    }

    let mut links_broken = false;

    for event in room_events.read() {
        let AssetEvent::Modified { id } = event else {
            continue;
        };

        if state.room_handles.iter().any(|h| h.id() == *id) {
            for (room_entity, marker) in &rooms_query {
                if marker.0 == *id {
                    commands
                        .entity(room_entity)
                        .insert(UpdateRoomGeometryMarker);
                    links_broken = true;
                }
            }
        }
    }
    if links_broken {
        for (trigger_entity, mut trigger) in &mut transition_triggers {
            if rooms_query.get(trigger.0).is_err() {
                for (room_entity, marker) in &rooms_query {
                    if room_entity == trigger.0
                        && let Some(config) = room_assets.get(marker.0)
                    {
                        commands
                            .entity(trigger_entity)
                            .insert(UnresolvedTargetMarker(config.name.clone()));
                    }
                }
                trigger.0 = Entity::PLACEHOLDER;
            }
        }
    }
}
