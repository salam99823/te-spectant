//! Spawn the main level.
use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    game::player::{PlayerAssets, player},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Option<Handle<AudioSource>>,
}

impl FromWorld for LevelAssets {
    fn from_world(_world: &mut World) -> Self {
        // let assets = world.resource::<AssetServer>();
        Self {
            music: None, // Some(assets.load("audio/music/GlobalMusic.ogg")),
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut level = commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![player(&player_assets, &mut texture_atlas_layouts),],
    ));
    if let Some(level_music) = &level_assets.music {
        level.with_children(|commands| {
            commands.spawn((Name::new("Music"), music(level_music.clone())));
        });
    }
}
