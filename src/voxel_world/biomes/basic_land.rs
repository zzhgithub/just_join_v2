// 基础大陆

use bevy::prelude::Vec3;
use ndshape::ConstShape;
use rand::Rng;

use crate::{
    tools::chunk_key_any_xyz_to_vec3,
    voxel_world::{
        chunk::ChunkKey,
        voxel::{AppleLeaf, AppleWood, Grass, Soli, Sown, Stone, Voxel, VoxelMaterial},
    },
};

use super::{
    find_out_chunk_keys, BiomesGenerator, SampleShape, TreeGentor, MOUNTAIN_LEVEL, SEE_LEVEL,
    SNOW_LEVEL,
};

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

    fn make_tree_with_info(
        &self,
        chunk_key: crate::voxel_world::chunk::ChunkKey,
        voxels: &mut Vec<Voxel>,
        _chunk_index: u32,
        _plane_index: u32,
        height: f32,
        xyz: [u32; 3],
    ) -> Option<(Vec<ChunkKey>, TreeGentor)> {
        let mut rng = rand::thread_rng();

        let root_pos = chunk_key_any_xyz_to_vec3(chunk_key, xyz);
        let leaf_center = root_pos + Vec3::new(0.0, 4.0, 0.0);
        // 判断 是否需要给其他的模块处理？
        let h = rng.gen_range(3..5);
        let r = rng.gen_range(2.9..4.6);
        // 这里 可以判断的又5个方向
        let mut tree_gentor = TreeGentor {
            tree: AppleWood::into_voxel(),
            leaf: if height >= -60. + 110. {
                Voxel::EMPTY
            } else {
                AppleLeaf::into_voxel()
            },
            trunk_params: (root_pos, h),
            leafs_params: (leaf_center, r, 0.0),
        };
        tree_gentor.make_tree_for_chunk(voxels, chunk_key.clone());

        let vec_list = find_out_chunk_keys(xyz, chunk_key.clone(), h, r.ceil() as u32);
        if vec_list.len() > 0 {
            return Some((vec_list, tree_gentor));
        }
        None
    }
}
