use ndshape::{ConstShape, ConstShape3u32};

use crate::{voxel_world::voxel::Voxel, CHUNK_SIZE_U32};

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
