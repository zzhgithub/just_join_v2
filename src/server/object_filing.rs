// 物体掉落相关
use bevy::{
    prelude::{
        Commands, Component, Entity, Event, EventReader, Plugin, ResMut, Resource, Transform,
        Update, Vec3,
    },
    transform::TransformBundle,
};
use bevy_rapier3d::prelude::{Collider, ColliderMassProperties, LockedAxes, RigidBody, Sleeping};

use crate::{staff::Staff, voxel_world::chunk::ChunkKey};

#[derive(Debug, Event)]
pub struct ObjectFillEvent {
    pub chunk_key: ChunkKey,
    pub xyz: [u32; 3],
    pub center: Vec3,
    pub staff: Staff,
}

// 掉落物
#[derive(Debug, Component)]
pub struct FilledObject;

// 掉落物池
#[derive(Debug, Resource, Clone)]
pub struct ObjectFilingManager {
    pub entities: Vec<Entity>,
}

pub struct ObjectFilingPlugin;

impl Plugin for ObjectFilingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<ObjectFillEvent>();
        app.insert_resource(ObjectFilingManager {
            entities: Vec::new(),
        });
        app.add_systems(Update, deal_object_filing);
    }
}

// 处理服务端的物体掉落
fn deal_object_filing(
    mut commands: Commands,
    mut fill_event: EventReader<ObjectFillEvent>,
    mut object_filing_manager: ResMut<ObjectFilingManager>,
) {
    for event in fill_event.iter() {
        // 通过staff 生成不同物体的加载模式
        match event.staff.staff_type {
            crate::staff::StaffType::Voxel(_) => {
                // 渲染一个正方形的 并且添加物理引擎
                let object = commands
                    .spawn(FilledObject)
                    .insert(Collider::cuboid(0.1, 0.1, 0.1))
                    .insert(RigidBody::Dynamic)
                    .insert(Sleeping::default())
                    .insert(ColliderMassProperties::Mass(300.0))
                    .insert(LockedAxes::ROTATION_LOCKED)
                    .insert(TransformBundle {
                        local: Transform::from_xyz(event.center.x, event.center.y, event.center.z),
                        ..Default::default()
                    })
                    .id();
                object_filing_manager.entities.push(object);
            }
            _ => {}
        }
    }
}
