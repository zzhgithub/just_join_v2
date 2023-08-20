use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

use crate::voxel_world::{chunk::ChunkKey, voxel::Voxel};

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ChunkQuery {
    // 获取全部的ChunkKey 的数据
    GetFullY(ChunkKey),
    // 更新某块数据
    Change {
        chunk_key: ChunkKey,
        pos: [u32; 3],
        voxel_type: Voxel,
    },
}
