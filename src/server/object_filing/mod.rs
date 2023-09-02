// 物体掉落相关
use bevy::{
    prelude::{
        Commands, Component, Entity, Event, EventReader, IntoSystemConfigs, Plugin, Query, Res,
        ResMut, Transform, Update, Vec3,
    },
    transform::TransformBundle,
    utils::HashMap,
};
use bevy_rapier3d::prelude::{
    Ccd, Collider, ColliderMassProperties, CollisionGroups, Group, LockedAxes, RigidBody, Sleeping,
};
use bevy_renet::renet::RenetServer;
use rand::Rng;

use crate::{
    common::ServerClipSpheres,
    staff::{Staff, StaffInfoStroge},
    tools::vec3_to_chunk_key_any_xyz,
    voxel_world::{
        chunk::{find_chunk_keys_array_by_shpere, generate_offset_array, ChunkKey},
        map_database::MapDataBase,
    },
    PY_DISTANCE,
};

use self::follow::ObjectFilingFollowPlugin;

use super::{
    message_def::{filled_object_message::FilledObjectMessage, ServerChannel},
    terrain_physics::ColliderSystem,
};

pub mod follow;
pub mod put_object;

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

pub trait GetChunkKey {
    fn get_chunk_key(&self) -> ChunkKey;
}

impl GetChunkKey for FilledObject {
    fn get_chunk_key(&self) -> ChunkKey {
        self.chunk_key.clone()
    }
}
pub struct ObjectFilingPlugin;

impl Plugin for ObjectFilingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<ObjectFillEvent>();
        app.add_plugins(ObjectFilingFollowPlugin);
        app.add_systems(
            Update,
            (
                deal_object_filing,
                update_filled_object_chunk_key,
                sync_filled_object_to_client,
                load_filled.after(ColliderSystem::ColliderSpawn),
                save_filled.before(ColliderSystem::ColliderDespawn),
            )
                .chain(),
        );
    }
}

// 处理服务端的物体掉落
fn deal_object_filing(mut commands: Commands, mut fill_event: EventReader<ObjectFillEvent>) {
    for event in fill_event.iter() {
        // 通过staff 生成不同物体的加载模式
        match event.staff.staff_type {
            crate::staff::StaffType::Voxel(_) => {
                // 渲染一个正方形的 并且添加物理引擎
                gen_filled_object(
                    &mut commands,
                    event.chunk_key,
                    event.center,
                    event.staff.clone(),
                );
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

fn map_chunk_key_filled_object<T>(
    query: &Query<(Entity, &T, &Transform)>,
) -> HashMap<ChunkKey, Vec<(Entity, T, Transform)>>
where
    T: Component + GetChunkKey + Clone,
{
    let mut hashed_object: HashMap<ChunkKey, Vec<(Entity, T, Transform)>> = HashMap::new();
    for (entity, filled_object, trf) in query.iter() {
        hashed_object
            .entry(filled_object.get_chunk_key())
            .or_insert(Vec::new())
            .push((entity.clone(), filled_object.clone(), trf.clone()));
    }

    hashed_object
}

// 物体的位置和信息同步到客户端
fn sync_filled_object_to_client(
    server_clip_spheres: Res<ServerClipSpheres>,
    query: Query<(Entity, &FilledObject, &Transform)>,
    mut server: ResMut<RenetServer>,
) {
    // 掉落物体和区块的相关配置
    let hashed_object = map_chunk_key_filled_object(&query);
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

// 掉落物进入存储和加载到存储

fn gen_filled_object(
    commands: &mut Commands,
    chunk_key: ChunkKey,
    center: Vec3,
    staff: Staff,
) -> Entity {
    let mut rng = rand::thread_rng();

    commands
        .spawn(FilledObject {
            chunk_key: chunk_key,
            staff: staff,
        })
        .insert(Collider::cuboid(0.05, 0.05, 0.05))
        .insert(RigidBody::Dynamic)
        .insert(Sleeping::default())
        .insert(ColliderMassProperties::Mass(300.0))
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Ccd::enabled())
        .insert(CollisionGroups::new(Group::GROUP_2, Group::GROUP_1))
        .insert(TransformBundle {
            // 添加随机偏移
            local: Transform::from_xyz(
                center.x + rng.gen_range(-0.05..=0.05),
                center.y + rng.gen_range(-0.05..=0.05),
                center.z + rng.gen_range(-0.05..=0.05),
            ),
            ..Default::default()
        })
        .id()
}

#[derive(Debug, Component, Clone)]

pub struct NeedSave {
    pub chunk_key: ChunkKey,
    pub id: usize,
}

impl GetChunkKey for NeedSave {
    fn get_chunk_key(&self) -> ChunkKey {
        self.chunk_key.clone()
    }
}

// 加载地图中的数据
fn load_filled(
    mut commands: Commands,
    server_clip_spheres: Res<ServerClipSpheres>,
    db: ResMut<MapDataBase>,
    query: Query<(Entity, &FilledObject, &Transform)>,
    staff_info_stroge: Res<StaffInfoStroge>,
) {
    let mut hashed_object = map_chunk_key_filled_object(&query);
    for (_, clip_spheres) in server_clip_spheres.clip_spheres.iter() {
        for chunk_key in find_chunk_keys_array_by_shpere(
            clip_spheres.new_sphere,
            generate_offset_array(PY_DISTANCE),
        )
        .drain(..)
        {
            hashed_object.remove(&chunk_key);
            let key = format!("FILL:{:?}", chunk_key);
            if let Ok(data) = db.db.remove(key.clone()) {
                if let Some(data) = data {
                    let data: Vec<(usize, [f32; 3])> = bincode::deserialize(&data).unwrap();
                    for (staff_id, pos) in data {
                        if let Some(staff) = staff_info_stroge.get(staff_id) {
                            gen_filled_object(&mut commands, chunk_key, Vec3::from(pos), staff);
                        }
                    }
                }
            }
        }
    }
    // 给要缓存的数据打上标签！
    for (chunk_key, vec_list) in hashed_object {
        for (entity, filled_object, _) in vec_list {
            commands.entity(entity).insert(NeedSave {
                chunk_key,
                id: filled_object.staff.id,
            });
        }
    }
}

fn save_filled(
    mut commands: Commands,
    db: ResMut<MapDataBase>,
    query: Query<(Entity, &NeedSave, &Transform)>,
) {
    let hashed_object = map_chunk_key_filled_object(&query);
    for (chunk_key, vec_list) in hashed_object {
        let key = format!("FILL:{:?}", chunk_key);
        // 数据保存进行数据库
        let data: Vec<(usize, [f32; 3])> = vec_list
            .clone()
            .iter()
            .map(|(_, need_save, trf)| {
                (
                    need_save.id,
                    [trf.translation.x, trf.translation.y, trf.translation.z],
                )
            })
            .collect();
        if let Ok(_) = db.db.insert(key, bincode::serialize(&data).unwrap()) {
            for (entity, _, _) in vec_list {
                commands.entity(entity).despawn();
            }
        }
    }
}
