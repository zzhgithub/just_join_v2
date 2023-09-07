// 基础大陆

use ndshape::ConstShape;

use crate::voxel_world::voxel::{Grass, Soli, Sown, Stone, Voxel, VoxelMaterial};

use super::{BiomesGenerator, SampleShape, MOUNTAIN_LEVEL, SEE_LEVEL, SNOW_LEVEL};

// 基础大陆
// 1. 雪顶
// 2. 石块
// 3. 草
// 4. 泥土 5
pub struct BasicLandBiomes;

impl BiomesGenerator for BasicLandBiomes {
    fn gen_land_with_info(
        &self,
        _chunk_key: crate::voxel_world::chunk::ChunkKey,
        voxels: &mut Vec<Voxel>,
        chunk_index: u32,
        _plane_index: u32,
        height: f32,
        xyz: [u32; 3],
    ) {
        let [x, y, z] = xyz;
        if height >= SNOW_LEVEL {
            // 雪线之上
            voxels[chunk_index as usize] = Sown::into_voxel();
            if y > 0 {
                // 雪之下雪乃
                let under_sown = SampleShape::linearize([x, y - 1, z]);
                voxels[under_sown as usize] = Sown::into_voxel();
            }
        } else if height >= MOUNTAIN_LEVEL {
            voxels[chunk_index as usize] = Stone::into_voxel();
            // 一层实体
        } else if height >= SEE_LEVEL {
            // 一层 草 5层的土
            voxels[chunk_index as usize] = Grass::into_voxel();
            for y_offset in 1..=5 {
                if y > y_offset {
                    let under_grass = SampleShape::linearize([x, y - y_offset, z]);
                    voxels[under_grass as usize] = Soli::into_voxel();
                }
            }
        }
    }
}
