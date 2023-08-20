use bevy::prelude::Vec3;
use ndshape::{ConstShape, ConstShape3u32};

use crate::{
    voxel_world::{
        chunk::{get_chunk_key_i3_by_vec3, ChunkKey},
        voxel::Voxel,
    },
    CHUNK_SIZE, CHUNK_SIZE_U32,
};

pub mod inspector_egui;

pub fn all_empty(voxels: &Vec<Voxel>) -> bool {
    for ele in voxels.iter() {
        if ele.id != 0 {
            return false;
        }
    }
    true
}

/**
 * 获取完全空的区块数据
 * Get all empty chunk data
 */
pub fn get_empty_chunk() -> Vec<Voxel> {
    let mut voxels = Vec::new();
    type SampleShape = ConstShape3u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32, CHUNK_SIZE_U32>;
    for _ in 0..SampleShape::SIZE {
        voxels.push(Voxel::EMPTY);
    }
    voxels
}

/**
 * 获取cube中心点 属于的 chunkKey和x y z坐标
 */
pub fn vec3_to_chunk_key_any_xyz(pos: Vec3) -> (ChunkKey, [u32; 3]) {
    // println!("此时的位置是: {:?}", pos);
    let chunk_key = ChunkKey(get_chunk_key_i3_by_vec3(pos));
    let x = (pos.x - (chunk_key.0.x * CHUNK_SIZE) as f32 + CHUNK_SIZE as f32 / 2. - 0.5) as u32;
    let y = (pos.y - (chunk_key.0.y * CHUNK_SIZE) as f32 + CHUNK_SIZE as f32 / 2. - 0.5) as u32;
    let z = (pos.z - (chunk_key.0.z * CHUNK_SIZE) as f32 + CHUNK_SIZE as f32 / 2. - 0.5) as u32;

    return (chunk_key, [x, y, z]);
}
