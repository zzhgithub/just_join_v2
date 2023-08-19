use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

use crate::voxel_world::chunk::ChunkKey;

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ChunkQuery {
    // 获取全部的ChunkKey 的数据
    GetFullY(ChunkKey),
}
