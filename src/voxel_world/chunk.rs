use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use bevy::{
    prelude::{IVec3, Resource, Vec3},
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

use crate::{common::Sphere3, CHUNK_SIZE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub struct ChunkKey(pub IVec3);

impl ChunkKey {
    pub fn add_ivec3(&self, key: IVec3) -> ChunkKey {
        let ivec3 = self.0.clone();
        ChunkKey(ivec3 + key)
    }

    pub fn as_u8_array(&self) -> [u8; 8] {
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        let hash_value = hasher.finish();
        hash_value.to_ne_bytes() // unsafe { std::mem::transmute(hash_value) };
    }
}

// 生成 y 偏移两为0的移动
pub fn generate_offset_array_with_y_0(chunk_distance: i32) -> Vec<IVec3> {
    let mut offsets = Vec::new();
    for x in -chunk_distance..=chunk_distance {
        for z in -chunk_distance..=chunk_distance {
            offsets.push(IVec3::new(x, 0, z));
        }
    }
    offsets
}

pub fn generate_offset_array(chunk_distance: i32) -> Vec<IVec3> {
    let mut offsets = Vec::new();
    for x in -chunk_distance..=chunk_distance {
        for y in -chunk_distance..=chunk_distance {
            for z in -chunk_distance..=chunk_distance {
                offsets.push(IVec3::new(x, y, z));
            }
        }
    }
    offsets.push(IVec3::ZERO);
    offsets
}

#[derive(Debug, Resource, Clone)]
pub struct NeighbourOffest(pub Vec<IVec3>);

pub fn generate_offset_resoure(radius: f32) -> NeighbourOffest {
    let chunk_distance = radius as i32 / CHUNK_SIZE;
    let mut offsets = generate_offset_array_with_y_0(chunk_distance);
    // itself
    offsets.push(IVec3::ZERO);

    NeighbourOffest(offsets)
}
pub fn generate_offset_resoure_min_1(radius: f32) -> NeighbourOffest {
    let mut chunk_distance = radius as i32 / CHUNK_SIZE;
    chunk_distance -= 1;

    let mut offsets = if chunk_distance > 0 {
        generate_offset_array_with_y_0(chunk_distance)
    } else {
        Vec::new()
    };
    // itself
    offsets.push(IVec3::ZERO);

    NeighbourOffest(offsets)
}

pub fn find_chunk_keys_array_by_shpere_y_0(sphere: Sphere3, offsets: Vec<IVec3>) -> Vec<ChunkKey> {
    let mut center_chunk_point = get_chunk_key_i3_by_vec3(sphere.center);
    center_chunk_point.y = 0;
    offsets
        .iter()
        .map(|&ele| ChunkKey(center_chunk_point + ele))
        .collect()
}

pub fn find_chunk_keys_array_by_shpere(sphere: Sphere3, offsets: Vec<IVec3>) -> Vec<ChunkKey> {
    let center_chunk_point = get_chunk_key_i3_by_vec3(sphere.center);
    offsets
        .iter()
        .map(|&ele| ChunkKey(center_chunk_point + ele))
        .collect()
}

// offsets 已经改变成了平面为零的情况 数据需要扩展的是y轴c
pub fn find_chunk_keys_by_shpere_to_full_height(
    sphere: Sphere3,
    offsets: Vec<IVec3>,
    mut rt: impl FnMut(ChunkKey),
) {
    let mut center_chunk_point = get_chunk_key_i3_by_vec3(sphere.center);
    center_chunk_point.y = 0;
    for &ele in offsets.iter() {
        for y_offset in -7..=8 {
            rt(ChunkKey(
                center_chunk_point
                    + ele
                    + IVec3 {
                        x: 0,
                        y: y_offset,
                        z: 0,
                    },
            ))
        }
    }
}

// 新版获取当前的 chunk_Key
pub fn get_chunk_key_i3_by_vec3(point: Vec3) -> IVec3 {
    IVec3 {
        x: get_chunk_key_axis(point.x),
        y: get_chunk_key_axis(point.y),
        z: get_chunk_key_axis(point.z),
    }
}

fn get_chunk_key_axis(x: f32) -> i32 {
    let x_dot = x * 10.;
    if x > 8. {
        return ((x_dot - (CHUNK_SIZE as f32 / 2.0) * 10.) as i32 / (CHUNK_SIZE * 10)) + 1;
    }
    if x < -8. {
        return ((x_dot + (CHUNK_SIZE as f32 / 2.0) * 10.) as i32 / (CHUNK_SIZE * 10)) - 1;
    }
    0
}
