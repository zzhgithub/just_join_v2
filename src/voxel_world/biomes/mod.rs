use ndshape::{ConstShape, ConstShape2u32, ConstShape3u32};
use noise::{
    core::worley::{distance_functions::euclidean, ReturnType},
    utils::NoiseMapBuilder,
    Worley,
};

use crate::{CHUNK_SIZE, CHUNK_SIZE_U32};

use super::{
    chunk::ChunkKey,
    voxel::{BuleGrass, DryGrass, Grass, Sand, Sown, Voxel, VoxelMaterial},
};

// 处理 生物群落
pub fn biomes_generate(
    chunk_key: ChunkKey,
    seed: i32,
    suface_index: Vec<u32>,
    voxels: &mut Vec<Voxel>,
) {
    type SampleShape = ConstShape3u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32, CHUNK_SIZE_U32>;
    type PanleShap = ConstShape2u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32>;

    if suface_index.len() == 0 {
        return;
    }
    // 生成噪声
    let noise = biomes_noise(chunk_key, seed);

    for index in suface_index {
        // 由噪声生产的特征值
        let [x, _, z] = SampleShape::delinearize(index);
        let index_2d = PanleShap::linearize([x, z]);
        let atrr = noise[index_2d as usize];
        if voxels[index as usize].id != Sown::ID {
            voxels[index as usize] = f_to_v(atrr);
        }
    }
}

// Tmp: 临时测试代码
fn f_to_v(data: f32) -> Voxel {
    if data < 0.1 {
        return Grass::into_voxel();
    } else if data < 0.4 {
        return DryGrass::into_voxel();
    } else if data < 0.6 {
        return Sown::into_voxel();
    } else if data < 0.8 {
        return Sand::into_voxel();
    } else {
        return BuleGrass::into_voxel();
    }
}

pub fn biomes_noise(chunk_key: ChunkKey, seed: i32) -> Vec<f32> {
    let noise = Worley::new(seed as u32)
        .set_distance_function(euclidean)
        .set_return_type(ReturnType::Value)
        .set_frequency(0.008);

    let x_offset = (chunk_key.0.x * CHUNK_SIZE) as f64;
    let z_offset = (chunk_key.0.z * CHUNK_SIZE) as f64;

    noise::utils::PlaneMapBuilder::<_, 2>::new(noise)
        .set_size(CHUNK_SIZE as usize, CHUNK_SIZE as usize)
        .set_x_bounds(x_offset, x_offset + CHUNK_SIZE as f64)
        .set_y_bounds(z_offset, z_offset + CHUNK_SIZE as f64)
        .build()
        .into_iter()
        .map(|x| x as f32)
        .collect()
}
