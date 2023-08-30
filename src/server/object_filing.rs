// 物体掉落相关
use bevy::{
    prelude::{
        Commands, Component, Entity, Event, EventReader, IntoSystemConfigs, Plugin, Query, Res,
        ResMut, Resource, Transform, Update, Vec3,
    },
    transform::TransformBundle,
    utils::HashMap,
};
use bevy_rapier3d::prelude::{
    Ccd, Collider, ColliderMassProperties, LockedAxes, RigidBody, Sleeping,
};
use bevy_renet::renet::RenetServer;

use crate::{
    common::ServerClipSpheres,
    staff::Staff,
    tools::vec3_to_chunk_key_any_xyz,
    voxel_world::chunk::{find_chunk_keys_array_by_shpere, generate_offset_array, ChunkKey},
    PY_DISTANCE,
};

use super::message_def::{filled_object_message::FilledObjectMessage, ServerChannel};

#[derive(Debug, Event)]
pub struct ObjectFillEvent {
    pub chunk_key: ChunkKey,
    pub xyz: [u32; 3],
    pub center: Vec3,
    pub staff: Staff,
}

// 掉落物
#[derive(Debug, Component, Clone)]
pub struct FilledObject {
    pub chunk_key: ChunkKey,
    pub staff: Staff,
}

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
        app.add_systems(
            Update,
            (
                deal_object_filing,
                update_filled_object_chunk_key,
                sync_filled_object_to_client,
            )
                .chain(),
        );
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
                    .spawn(FilledObject {
                        chunk_key: event.chunk_key,
                        staff: event.staff.clone(),
                    })
                    .insert(Collider::cuboid(0.05, 0.05, 0.05))
                    .insert(RigidBody::Dynamic)
                    .insert(Sleeping::default())
                    .insert(ColliderMassProperties::Mass(300.0))
                    .insert(LockedAxes::ROTATION_LOCKED)
                    .insert(Ccd::enabled())
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

// 同步数据每个时刻的 位移信息
fn update_filled_object_chunk_key(mut query: Query<(&mut FilledObject, &Transform)>) {
    for (mut obj, trf) in query.iter_mut() {
        let (chunk_key, _) = vec3_to_chunk_key_any_xyz(trf.translation);
        obj.chunk_key = chunk_key;
    }
}

// 物体的位置和信息同步到客户端
fn sync_filled_object_to_client(
    server_clip_spheres: Res<ServerClipSpheres>,
    query: Query<(Entity, &FilledObject, &Transform)>,
    mut server: ResMut<RenetServer>,
) {
    // 掉落物体和区块的相关配置
    let mut hashed_object: HashMap<ChunkKey, Vec<(Entity, FilledObject, Transform)>> =
        HashMap::new();
    for (entity, filled_object, trf) in query.iter() {
        hashed_object
            .entry(filled_object.chunk_key)
            .or_insert(Vec::new())
            .push((entity.clone(), filled_object.clone(), trf.clone()));
    }
    for (client_id, clip_spheres) in server_clip_spheres.clip_spheres.iter() {
        let mut staff_list: Vec<(Entity, usize, [f32; 3])> = Vec::new();
        // 对每个球体展开一阶
        for chunk_key in find_chunk_keys_array_by_shpere(
            clip_spheres.new_sphere,
            generate_offset_array(PY_DISTANCE),
        )
        .drain(..)
        {
            if let Some(ele) = hashed_object.get(&chunk_key) {
                for (entity, filled_object, trf) in ele.iter() {
                    staff_list.push((
                        entity.clone(),
                        filled_object.staff.id.clone(),
                        [trf.translation.x, trf.translation.y, trf.translation.z],
                    ));
                }
            }
        }
        let message;
        if !staff_list.is_empty() {
            // FIXME: 处理掉落物过多的问题？
            message =
                bincode::serialize(&FilledObjectMessage::SyncFilledObject(staff_list)).unwrap();
        } else {
            message =
                bincode::serialize(&FilledObjectMessage::SyncFilledObject(Vec::new())).unwrap();
        }
        server.send_message(*client_id, ServerChannel::FilledObjectMessage, message);
    }
}

//TODO 掉落物 进存储？
// 掉落物 从存储中获取？
