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
pub mod string;
pub mod zone;

pub fn all_empty(voxels: &[Voxel]) -> bool {
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
 * 获取完全空的区块数据
 * Get all empty chunk data
 */
pub fn get_all_v_chunk(voxel: Voxel) -> Vec<Voxel> {
    let mut voxels = Vec::new();
    type SampleShape = ConstShape3u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32, CHUNK_SIZE_U32>;
    for _ in 0..SampleShape::SIZE {
        voxels.push(voxel.clone());
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

    (chunk_key, [x, y, z])
}

//  chunkKey和x y z坐标 还原到 Cube的中心位置
pub fn chunk_key_any_xyz_to_vec3(chunk_key: ChunkKey, xyz: [u32; 3]) -> Vec3 {
    let x =
        (chunk_key.0.x as f32) * (CHUNK_SIZE as f32) - CHUNK_SIZE as f32 / 2. + xyz[0] as f32 + 0.5;
    let y =
        (chunk_key.0.y as f32) * (CHUNK_SIZE as f32) - CHUNK_SIZE as f32 / 2. + xyz[1] as f32 + 0.5;
    let z =
        (chunk_key.0.z as f32) * (CHUNK_SIZE as f32) - CHUNK_SIZE as f32 / 2. + xyz[2] as f32 + 0.5;
    Vec3::new(x, y, z)
}

// 计算点所在的 方块终点是什么
pub fn pos_to_center(pos: Vec3) -> Vec3 {
    let res = Vec3 {
        x: pos.x.floor(),
        y: pos.y.floor(),
        z: pos.z.floor(),
    };
    res + Vec3::splat(0.5)
}
