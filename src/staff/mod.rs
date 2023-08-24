use bevy::{
    prelude::{warn, App, AssetServer, Commands, Handle, Image, Plugin, Res, Resource, Startup},
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
    // 如果可以的 怎么生成体素
    pub voxel: Option<Voxel>,
}

#[derive(Debug, Resource)]
pub struct StaffInfoStroge {
    pub data: HashMap<usize, Staff>,
}

impl StaffInfoStroge {
    fn register(&mut self, staff: Staff) {
        if self.data.contains_key(&staff.id) {
            warn!("{} is already registered", staff.id);
        }
        self.data.insert(staff.id, staff);
    }
}

pub struct StaffInfoPlugin;

impl Plugin for StaffInfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut stroge = StaffInfoStroge {
        data: HashMap::default(),
    };
    stroge.register(Staff {
        id: 0,
        name: String::from("Stone"),
        icon: asset_server.load("textures/002.png"),
        voxel: Some(Stone::into_voxel()),
    });
    stroge.register(Staff {
        id: 1,
        name: String::from("Grass"),
        icon: asset_server.load("textures/草坪.png"),
        voxel: Some(Grass::into_voxel()),
    });
    stroge.register(Staff {
        id: 2,
        name: String::from("Soli"),
        icon: asset_server.load("textures/003.png"),
        voxel: Some(Soli::into_voxel()),
    });
    stroge.register(Staff {
        id: 3,
        name: String::from("Sand"),
        icon: asset_server.load("textures/沙子.png"),
        voxel: Some(Sand::into_voxel()),
    });
    stroge.register(Staff {
        id: 4,
        name: String::from("Sown"),
        icon: asset_server.load("textures/雪.png"),
        voxel: Some(Sown::into_voxel()),
    });

    commands.insert_resource(stroge);
}
