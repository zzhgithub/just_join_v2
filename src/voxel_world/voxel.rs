use bevy::reflect::Reflect;
use block_mesh::{MergeVoxel, Voxel as MeshVoxel, VoxelVisibility};
use serde::{Deserialize, Serialize};

/**
 * 体素类型
 */
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, Reflect)]
pub struct Voxel {
    pub id: u8,
}

impl Voxel {
    pub const EMPTY: Self = Self { id: 0 };
    pub const FILLED: Self = Self { id: 1 };
}

impl MeshVoxel for Voxel {
    fn get_visibility(&self) -> VoxelVisibility {
        // 这里过滤掉水
        if self.id > 0 && self.id != 5 {
            return VoxelVisibility::Opaque;
        }
        VoxelVisibility::Empty
    }
}

impl MergeVoxel for Voxel {
    type MergeValue = u8;

    fn merge_value(&self) -> Self::MergeValue {
        self.id
    }
}

pub trait VoxelMaterial {
    const ID: u8;

    fn into_voxel() -> Voxel {
        Voxel { id: Self::ID }
    }
}

// 用来生成材质宏
#[macro_export]
macro_rules! voxel_material {
    ($types: ident,$ch_name: ident,$id: expr) => {
        pub struct $types;
        impl $types {
            pub const NAME: &'static str = stringify!($types);
            pub const CN_NAME: &'static str = stringify!($ch_name);
        }
        impl $crate::voxel_world::voxel::VoxelMaterial for $types {
            const ID: u8 = $id;
        }
    };
}

// 定义有材质的体素模型
voxel_material!(Empty, 空气, 0);
voxel_material!(Stone, 岩石块, 1);
voxel_material!(Soli, 土壤, 2);
voxel_material!(Grass, 草方块, 3);
voxel_material!(Sown, 雪方块, 4);
voxel_material!(Water, 水, 5);
voxel_material!(Sand, 沙子, 6);
voxel_material!(BasicStone, 基岩, 7);
