// 苍翠大陆

use ndshape::ConstShape;

use crate::voxel_world::voxel::{BuleGrass, Soli, Sown, Stone, VoxelMaterial};

use super::{BiomesGenerator, SampleShape, MOUNTAIN_LEVEL, SEE_LEVEL, SNOW_LEVEL};

pub struct BuleLandBoimes;

impl BiomesGenerator for BuleLandBoimes {
    fn gen_land_with_info(
        &self,
        _chunk_key: crate::voxel_world::chunk::ChunkKey,
        voxels: &mut Vec<crate::voxel_world::voxel::Voxel>,
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
        } else if height >= MOUNTAIN_LEVEL + 6.0 {
            voxels[chunk_index as usize] = Stone::into_voxel();
            // 一层实体
        } else if height >= SEE_LEVEL {
            // 一层 草 5层的土
            voxels[chunk_index as usize] = BuleGrass::into_voxel();
            for y_offset in 1..=5 {
                if y > y_offset {
                    let under_grass = SampleShape::linearize([x, y - y_offset, z]);
                    voxels[under_grass as usize] = Soli::into_voxel();
                }
            }
        }
    }
}
