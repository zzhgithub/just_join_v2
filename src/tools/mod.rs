use crate::voxel_world::voxel::Voxel;

pub mod inspector_egui;

pub fn all_empty(voxels: Vec<Voxel>) -> bool {
    for ele in voxels.iter() {
        if ele.id != 0 {
            return false;
        }
    }
    true
}
