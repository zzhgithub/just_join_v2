use std::f32::consts::PI;

use bevy::{prelude::Quat, reflect::Reflect};
use block_mesh::{MergeVoxel, Voxel as MeshVoxel, VoxelVisibility};
use serde::{Deserialize, Serialize};

use super::voxel_mesh::VOXEL_MESH_MAP;

/**
 * 体素类型
 *
 * 这里要设计使用 u32的数据
 * voxel_data >> 8u & 0b111 8位后的数据 9 10 11位置的数据
 * voxel_data & 255u 表示取最后的8位
 * 方块是否透明是否要记录呢？不用在数据库中，但是转回来的时候我要知道。并且可以生成对应的mesh在地图中
 *
 * 存储类型：
 * 体素类型  方块方向
 * [0-8]   [9 10]
 * todo 在某种情况下计算不同位置的 图片索引和贴图？
 *
 * 展示时的数据
 * 贴图索引  法向量  方块方向
 * [0-8]   [9-11] [12 13]
 *
 * 是否可视等 是在其他地方定义的
 *
 */
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, Reflect, PartialEq, Eq, Hash)]
pub struct Voxel {
    pub id: u8,
    pub direction: VoxelDirection,
}

// 体素方向
#[derive(Debug, Clone, Copy, Reflect, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub enum VoxelDirection {
    #[default]
    Z,
    NZ, // 指向Z的负数半轴
    X,
    NX, // 指向X的负数半轴
}

impl VoxelDirection {
    pub fn to_quat(&self) -> Quat {
        match self {
            VoxelDirection::Z => Quat::from_rotation_y(0.0),
            VoxelDirection::X => Quat::from_rotation_y(PI / 2.0),
            VoxelDirection::NZ => Quat::from_rotation_y(PI),
            VoxelDirection::NX => Quat::from_rotation_y(3.0 * PI / 2.0),
        }
    }
}

pub const VOXEL_DIRECTION_VEC: [VoxelDirection; 4] = [
    VoxelDirection::Z,
    VoxelDirection::NZ,
    VoxelDirection::X,
    VoxelDirection::NX,
];

impl PartialOrd for Voxel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for Voxel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl Voxel {
    pub const EMPTY: Self = Self {
        id: 0,
        direction: VoxelDirection::Z,
    };
    pub const FILLED: Self = Self {
        id: 1,
        direction: VoxelDirection::Z,
    };

    // 转换成32位的存在类型
    pub fn into_save_u32(&self) -> u32 {
        let direction_base = match self.direction {
            VoxelDirection::Z => 0u32 << 8u32,
            VoxelDirection::NZ => 1u32 << 8u32,
            VoxelDirection::X => 2u32 << 8u32,
            VoxelDirection::NX => 3u32 << 8u32,
        };
        (self.id as u32) | direction_base
    }

    // u32 位置类型转换成 体素类型
    pub fn u32_into_voxel(data: u32) -> Self {
        Self {
            id: Self::pick_id(data),
            direction: Self::pick_direction(data),
        }
    }

    pub fn pick_id(data: u32) -> u8 {
        (data & 255u32) as u8
    }

    pub fn pick_direction(data: u32) -> VoxelDirection {
        VOXEL_DIRECTION_VEC[(data >> 8u32 & 0b11) as usize]
    }

    pub fn next_direction(&self) -> Self {
        Self {
            id: self.id,
            direction: match self.direction {
                VoxelDirection::Z => VoxelDirection::X,
                VoxelDirection::X => VoxelDirection::NZ,
                VoxelDirection::NZ => VoxelDirection::NX,
                VoxelDirection::NX => VoxelDirection::Z,
            },
        }
    }
}

impl MeshVoxel for Voxel {
    fn get_visibility(&self) -> VoxelVisibility {
        // 这里控制显示问题
        if VOXEL_MESH_MAP.contains_key(&self.id) {
            return VoxelVisibility::Empty;
        }
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
        Voxel {
            id: Self::ID,
            ..Default::default()
        }
    }

    // 转化成 有方向的体素数据
    fn into_voxel_with_dir(direction: VoxelDirection) -> Voxel {
        Voxel {
            id: Self::ID,
            direction,
        }
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
voxel_material!(DryGrass, 干草地, 8);
voxel_material!(BuleGrass, 苍翠地, 9);
voxel_material!(AppleWood, 苹果树原木, 10);
voxel_material!(AppleLeaf, 苹果树叶子, 11);
voxel_material!(TestCube, 测试方块, 12);
voxel_material!(WorkCube, 工作方块, 13);
