use bevy::{
    prelude::{
        error, warn, App, AssetServer, Handle, Image, IntoSystemConfigs, Plugin, Res, ResMut,
        Resource, Startup, SystemSet,
    },
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use crate::voxel_world::voxel::Voxel;

use self::rule::StaffRulePlugin;

pub mod rule;

#[derive(Debug, Clone)]
pub struct Staff {
    // 物品id
    pub id: usize,
    // 物品名称
    pub name: String,
    // 物品图示
    pub icon: Handle<Image>,
    // 物品类型
    pub staff_type: StaffType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StaffType {
    // 体素方块
    Voxel(Voxel),
    // 工具(staff id)
    Tool(usize),
    // 特殊的可放置的物体
    Sp(usize),
    // 消耗品
    Consumable(usize),
}

#[derive(Debug, Resource)]
pub struct StaffInfoStroge {
    pub data: HashMap<usize, Staff>,
    pub voxel_staff: HashMap<u8, Staff>,
}

impl StaffInfoStroge {
    fn register(&mut self, staff: Staff) {
        if self.data.contains_key(&staff.id) {
            warn!("{} is already registered", staff.id);
        }
        if let StaffType::Voxel(voxel) = staff.staff_type {
            self.voxel_staff.insert(voxel.id, staff.clone());
        }
        self.data.insert(staff.id, staff);
    }
    // 通过体素获取物品
    pub fn voxel_to_staff(&self, voxel: Voxel) -> Option<&Staff> {
        self.voxel_staff.get(&voxel.id)
    }
    // 通过 物品点 获取物品id
    pub fn get(&self, staff_id: usize) -> Option<Staff> {
        self.data.get(&staff_id).map(|a| a.clone())
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum StaffSet {
    Init,
}

pub struct StaffInfoPlugin;

impl Plugin for StaffInfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StaffRulePlugin);
        app.insert_resource(StaffInfoStroge {
            data: HashMap::default(),
            voxel_staff: HashMap::default(),
        });
        app.add_systems(Startup, setup.in_set(StaffSet::Init));
    }
}

fn setup(mut storge: ResMut<StaffInfoStroge>, asset_server: Res<AssetServer>) {
    load_staff_configs(String::from("staff.ron"), &mut storge, Some(&asset_server));
}

pub struct ServerStaffInfoPlugin;

impl Plugin for ServerStaffInfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StaffRulePlugin);
        app.insert_resource(StaffInfoStroge {
            data: HashMap::default(),
            voxel_staff: HashMap::default(),
        });
        app.add_systems(Startup, server_setup.in_set(StaffSet::Init));
    }
}

fn server_setup(mut storge: ResMut<StaffInfoStroge>) {
    load_staff_configs(String::from("staff.ron"), &mut storge, None);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffMeta {
    id: usize,
    name: String,
    icon_string: String,
    staff_type: StaffType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffConfigs {
    pub configs: Vec<StaffMeta>,
}

fn load_staff_configs(
    path: String,
    staff_info_stroge: &mut StaffInfoStroge,
    asset_server: Option<&AssetServer>,
) {
    // 加载文件到数据
    match std::fs::File::open(path) {
        Ok(file) => {
            let res: StaffConfigs = ron::de::from_reader(file).unwrap();
            for mate in res.configs {
                if let Some(asset_server) = asset_server {
                    staff_info_stroge.register(Staff {
                        id: mate.id,
                        name: mate.name,
                        icon: asset_server.load(mate.icon_string),
                        staff_type: mate.staff_type,
                    });
                } else {
                    staff_info_stroge.register(Staff {
                        id: mate.id,
                        name: mate.name,
                        icon: Handle::default(),
                        staff_type: mate.staff_type,
                    });
                }
            }
        }
        Err(_) => {
            error!("读取Staff配置数据失败");
        }
    }
}
