use ndshape::ConstShape;

use crate::voxel_world::voxel::{Sown, VoxelMaterial};

use super::{BiomesGenerator, SampleShape};

// 雪原大陆
pub struct SnowLandBiomes;

impl BiomesGenerator for SnowLandBiomes {
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
            voxels[index as usize] = Sown::into_voxel();
        }
    }
}
