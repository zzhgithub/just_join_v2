// 特别的体素类型的展示
// 这个地方要给server和client使用

use bevy::{
    prelude::{AssetServer, Handle, Image, Mesh, Plugin, Res, ResMut, Resource, Startup},
    utils::HashMap,
};
use bevy_vox_mesh::VoxMeshPlugin;
use lazy_static::lazy_static;

use crate::voxel_world::voxel::{VoxelMaterial, WorkCube};

#[derive(Debug, Clone)]
pub struct VoxelMeshConfig {
    // 是否生成碰撞体
    pub collider: bool,
    // 模型列表
    pub vox_list: Vec<String>,
    // 图片列表
    pub image_list: Vec<String>,
}

lazy_static! {
    pub static ref VOXEL_MESH_MAP: HashMap<u8, VoxelMeshConfig> = {
        let mut map = HashMap::new();
        // 这里加上数据
        map.insert(WorkCube::into_voxel().id,VoxelMeshConfig{
            collider: true,
            vox_list: vec![String::from("vox/工作台.vox")],
            image_list: Vec::new(),
        });
        map
    };
}

#[derive(Debug, Clone)]
pub struct MeshMateData {
    pub vox_list: Vec<Handle<Mesh>>,
    pub image_list: Vec<Handle<Image>>,
}

#[derive(Debug, Clone, Resource)]
pub struct VoxelMeshStorge {
    pub data: HashMap<u8, MeshMateData>,
}

// 加载数据
pub struct VoxelMeshPlugin;

impl Plugin for VoxelMeshPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(VoxMeshPlugin::default());
        app.insert_resource(VoxelMeshStorge {
            data: HashMap::new(),
        });
        app.add_systems(Startup, init_mesh_resource);
    }
}

fn init_mesh_resource(assets: Res<AssetServer>, mut voxel_mesh_storge: ResMut<VoxelMeshStorge>) {
    for (key, config) in VOXEL_MESH_MAP.iter() {
        let mut vox_list: Vec<Handle<Mesh>> = Vec::new();
        for vox_str in config.vox_list.iter() {
            vox_list.push(assets.load(vox_str));
        }
        let mut image_list: Vec<Handle<Image>> = Vec::new();
        for image_str in config.image_list.iter() {
            image_list.push(assets.load(image_str));
        }
        voxel_mesh_storge.data.insert(
            *key,
            MeshMateData {
                vox_list: vox_list,
                image_list: image_list,
            },
        );
    }
}

