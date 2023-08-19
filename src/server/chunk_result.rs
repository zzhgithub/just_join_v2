use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

use crate::voxel_world::{chunk::ChunkKey, voxel::Voxel};

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ChunkResult {
    ChunkData {
        key: ChunkKey,
        data: Vec<Voxel>,
    },
    ChunkEmpty(ChunkKey),
}
