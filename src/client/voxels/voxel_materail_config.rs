use std::io::Write;

use bevy::{prelude::Resource, reflect::Reflect, utils::HashMap};
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::voxel_world::voxel::{Grass, Soli, Stone, VoxelMaterial};

#[derive(Debug, Clone, Serialize, Deserialize, Default, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct VoxelConfig {
    pub index: u32,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, InspectorOptions, Reflect)]
#[reflect(InspectorOptions)]
pub struct VoxelTypeConfig {
    pub type_name: String,
    pub type_ch_name: String,
    // 默认配置
    pub default: VoxelConfig,
    // 各个法向量下的配置
    pub normal: HashMap<u8, VoxelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Resource, InspectorOptions, Default, Reflect)]
pub struct MaterailConfiguration {
    // 体素类型列表
    pub voxels: HashMap<u8, VoxelTypeConfig>,
    // 文件地址列表
    pub files: Vec<String>,
}

#[macro_export]
macro_rules! add_volex {
    ($types: ident,$class: expr) => {
        if (!$class.voxels.contains_key(&$types::ID)) {
            $class.voxels.insert(
                $types::ID,
                VoxelTypeConfig {
                    type_name: String::from($types::NAME),
                    type_ch_name: String::from($types::CN_NAME),
                    ..Default::default()
                },
            );
            println!(
                "* 加载体素[{}][{}]",
                String::from($types::NAME),
                String::from($types::CN_NAME),
            );
        } else {
            println!(
                "加载体素[{}][{}]",
                String::from($types::NAME),
                String::from($types::CN_NAME),
            );
        }
    };
}

impl MaterailConfiguration {
    // 初始化
    pub fn new() -> Self {
        // 读取文件夹下的
        Self {
            voxels: HashMap::default(),
            files: Vec::new(),
        }
    }

    // 加载文件夹下的数据
    pub fn load_pic_files(mut self, dir: String) -> Self {
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let file_path = entry.path();
                // 在这里处理文件，例如打印文件路径

                let path = file_path.to_str().unwrap().replace("assets/", "");
                if self.files.contains(&path) {
                    self.files.push(path);
                    println!("* 文件路径: {}", file_path.display());
                } else {
                    println!("文件路径: {}", file_path.display());
                }
            }
        }
        self
    }

    pub fn load_all_voxels(mut self) -> Self {
        // 初始化全部的数据
        add_volex!(Stone, self);
        add_volex!(Soli, self);
        add_volex!(Grass, self);
        // todo 加载其他的类型
        self
    }

    pub fn read_file(self, path: String) -> Result<Self, ron::Error> {
        let reader = std::fs::File::open(path);
        match reader {
            Ok(file) => {
                // 如果成功取配置
                let res: Self = ron::de::from_reader(file).unwrap();
                let new_self = res.load_all_voxels();
                // new_self = self.load_pic_files(String::from("assets/textures"));
                Ok(new_self)
            }
            Err(_) => {
                print!("没有找配置文件第一次加载");
                let mut new_self = self.load_pic_files(String::from("assets/textures"));
                new_self = new_self.load_all_voxels();
                Ok(new_self)
            }
        }
    }

    pub fn write_file(self, path: String) {
        let res = ron::to_string(&self).unwrap();
        let mut file = std::fs::File::create(path).expect("create failed");
        file.write_all(res.as_bytes()).unwrap();
    }

    // 通过面 和 体素类型获取 图片的索引
    pub fn find_volex_index(self, normal: u8, volex_type: &u8) -> u32 {
        return match self.voxels.get(volex_type) {
            Some(config) => {
                return match config.normal.get(&normal) {
                    Some(vconfig) => vconfig.index,
                    None => config.default.index,
                };
            }
            None => 0,
        };
    }
}
