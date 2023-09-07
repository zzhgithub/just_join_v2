use ndshape::ConstShape;

use crate::voxel_world::voxel::{Sand, VoxelMaterial};

// 沙漠大陆
use super::{BiomesGenerator, SampleShape};

pub struct SandLandBiomes;

impl BiomesGenerator for SandLandBiomes {
    fn gen_land_with_info(
        &self,
        _chunk_key: crate::voxel_world::chunk::ChunkKey,
        voxels: &mut Vec<crate::voxel_world::voxel::Voxel>,
        _chunk_index: u32,
        _plane_index: u32,
        _height: f32,
        xyz: [u32; 3],
    ) {
        let [x, y, z] = xyz;
        for y_offset in 0..=y {
            let index = SampleShape::linearize([x, y - y_offset, z]);
            voxels[index as usize] = Sand::into_voxel();
        }
    }
}
