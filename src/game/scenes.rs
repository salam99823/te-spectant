use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    reflect::TypePath,
};
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::game::collision_groups::{PLAYER, WALL};

#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
#[require(
    Name::new("Wall"),
    CollisionGroups::new(WALL, PLAYER),
    SolverGroups::new(WALL, PLAYER)
)]
pub struct Wall;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WallConfig {
    pub half_extents: Vec2,
    pub position: Vec2,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TriggerConfig {
    pub name: String,
    pub target_room_name: String,
    pub half_extents: Vec2,
    pub position: Vec2,
}

#[derive(Asset, TypePath, Serialize, Deserialize, Clone, Debug)]
pub struct RoomConfig {
    pub name: String,
    pub image_path: String,
    pub translation: Vec2,
    pub walls: Vec<WallConfig>,
    pub triggers: Vec<TriggerConfig>,
}

#[derive(Asset, TypePath, Serialize, Deserialize, Clone, Debug)]
pub struct MansionConfig {
    pub rooms: Vec<String>,
}

#[derive(Default, TypePath)]
pub struct MansionRonLoader;

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("RON error: {0}")]
    Ron(#[from] ron::error::SpannedError),
}

impl AssetLoader for MansionRonLoader {
    type Asset = MansionConfig;
    type Settings = ();
    type Error = LoaderError;
    async fn load(
        &self,
        r: &mut dyn Reader,
        _: &(),
        _: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut b = Vec::new();
        r.read_to_end(&mut b).await?;
        let c: MansionConfig = ron::de::from_bytes(&b)?;
        Ok(c)
    }
    fn extensions(&self) -> &[&str] {
        &["mansion.ron"]
    }
}

#[derive(Default, TypePath)]
pub struct RoomRonLoader;
impl AssetLoader for RoomRonLoader {
    type Asset = RoomConfig;
    type Settings = ();
    type Error = LoaderError;
    async fn load(
        &self,
        r: &mut dyn Reader,
        _: &(),
        _: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut b = Vec::new();
        r.read_to_end(&mut b).await?;
        let c: RoomConfig = ron::de::from_bytes(&b)?;
        Ok(c)
    }
    fn extensions(&self) -> &[&str] {
        &["room.ron", "ron"]
    }
}
