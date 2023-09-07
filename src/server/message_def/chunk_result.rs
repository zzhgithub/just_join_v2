use bevy::prelude::Component;
use bit_vec::BitVec;
use huffman_compress::Tree;
use serde::{Deserialize, Serialize};

use crate::voxel_world::{chunk::ChunkKey, voxel::Voxel};

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ChunkResult {
    ChunkData {
        key: ChunkKey,
        data: (BitVec, Tree<Voxel>),
    },
    ChunkSame((ChunkKey,Voxel)),
    ChunkUpdateOne {
        chunk_key: ChunkKey,
        pos: [u32; 3],
        voxel_type: Voxel,
    },
}
