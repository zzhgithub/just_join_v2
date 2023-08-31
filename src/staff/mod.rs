use bevy::{
    prelude::{
        warn, App, AssetServer, Handle, Image, IntoSystemConfigs, Plugin, Res, ResMut, Resource,
        Startup, SystemSet,
    },
    utils::HashMap,
};

use crate::voxel_world::voxel::{Grass, Sand, Soli, Sown, Stone, Voxel, VoxelMaterial};

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

#[derive(Debug, Clone)]
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
        app.insert_resource(StaffInfoStroge {
            data: HashMap::default(),
            voxel_staff: HashMap::default(),
        });
        app.add_systems(Startup, setup.in_set(StaffSet::Init));
    }
}

fn setup(mut storge: ResMut<StaffInfoStroge>, asset_server: Res<AssetServer>) {
    storge.register(Staff {
        id: 0,
        name: String::from("Stone"),
        icon: asset_server.load("textures/002.png"),
        staff_type: StaffType::Voxel(Stone::into_voxel()),
    });
    storge.register(Staff {
        id: 1,
        name: String::from("Grass"),
        icon: asset_server.load("textures/草坪.png"),
        staff_type: StaffType::Voxel(Grass::into_voxel()),
    });
    storge.register(Staff {
        id: 2,
        name: String::from("Soli"),
        icon: asset_server.load("textures/003.png"),
        staff_type: StaffType::Voxel(Soli::into_voxel()),
    });
    storge.register(Staff {
        id: 3,
        name: String::from("Sand"),
        icon: asset_server.load("textures/沙子.png"),
        staff_type: StaffType::Voxel(Sand::into_voxel()),
    });
    storge.register(Staff {
        id: 4,
        name: String::from("Sown"),
        icon: asset_server.load("textures/雪.png"),
        staff_type: StaffType::Voxel(Sown::into_voxel()),
    });
}

pub struct ServerStaffInfoPlugin;

impl Plugin for ServerStaffInfoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StaffInfoStroge {
            data: HashMap::default(),
            voxel_staff: HashMap::default(),
        });
        app.add_systems(Startup, server_setup.in_set(StaffSet::Init));
    }
}

fn server_setup(mut storge: ResMut<StaffInfoStroge>) {
    storge.register(Staff {
        id: 0,
        name: String::from("Stone"),
        icon: Handle::default(),
        staff_type: StaffType::Voxel(Stone::into_voxel()),
    });
    storge.register(Staff {
        id: 1,
        name: String::from("Grass"),
        icon: Handle::default(),
        staff_type: StaffType::Voxel(Grass::into_voxel()),
    });
    storge.register(Staff {
        id: 2,
        name: String::from("Soli"),
        icon: Handle::default(),
        staff_type: StaffType::Voxel(Soli::into_voxel()),
    });
    storge.register(Staff {
        id: 3,
        name: String::from("Sand"),
        icon: Handle::default(),
        staff_type: StaffType::Voxel(Sand::into_voxel()),
    });
    storge.register(Staff {
        id: 4,
        name: String::from("Sown"),
        icon: Handle::default(),
        staff_type: StaffType::Voxel(Sown::into_voxel()),
    });
}
