use bevy::{
    prelude::{warn, Plugin, Res, ResMut, Resource, Update},
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_renet::renet::RenetServer;
use ndshape::{ConstShape, ConstShape3u32};

use crate::{
    client::{chunk_query::ChunkQuery, client_channel::ClientChannel},
    tools::all_empty,
    voxel_world::{
        chunk::ChunkKey,
        chunk_map::ChunkMap,
        map_database::{DbSaveTasks, MapDataBase},
        voxel::{BasicStone, VoxelMaterial},
    },
    CHUNK_SIZE, CHUNK_SIZE_U32,
};

use super::{
    chunk_result::ChunkResult,
    server_channel::ServerChannel,
    terrain_physics::{ColliderManager, ColliderTasksManager, ColliderUpdateTasksManager},
};

#[derive(Debug, Resource)]
pub struct ChunkResultTasks {
    pub tasks: Vec<Task<(u64, Vec<u8>)>>,
}

pub fn deal_chunk_query_system(
    mut server: ResMut<RenetServer>,
    mut chunk_map: ResMut<ChunkMap>,
    mut db_save_task: ResMut<DbSaveTasks>,
    mut db: ResMut<MapDataBase>,
    mut tasks: ResMut<ChunkResultTasks>,
    collider_manager: Res<ColliderManager>,
    mut collider_update_tasks_manager: ResMut<ColliderUpdateTasksManager>,
    mut collider_tasks: ResMut<ColliderTasksManager>,
) {
    let pool = AsyncComputeTaskPool::get();
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::ChunkQuery) {
            let chunk_query: ChunkQuery = bincode::deserialize(&message).unwrap();
            match chunk_query {
                ChunkQuery::GetFullY(chunk_key) => {
                    // 获取全部的值 然后返回
                    let last_inex = -128 / CHUNK_SIZE + 1;
                    for y_offset in last_inex..=128 / CHUNK_SIZE {
                        let mut new_key = chunk_key;
                        new_key.0.y = y_offset;
                        let voxels;
                        if let Some(data) = chunk_map.map_data.get(&new_key) {
                            voxels = data.clone();
                        } else {
                            voxels = db.find_by_chunk_key(new_key, db_save_task.as_mut());
                        }
                        let message = if all_empty(&voxels) {
                            bincode::serialize(&ChunkResult::ChunkEmpty(new_key)).unwrap()
                        } else {
                            bincode::serialize(&ChunkResult::ChunkData {
                                key: new_key,
                                data: voxels.clone(),
                            })
                            .unwrap()
                        };

                        let task = pool.spawn(async move { (client_id, message) });
                        tasks.tasks.push(task);
                    }
                }
                ChunkQuery::Change {
                    chunk_key,
                    pos,
                    voxel_type,
                } => {
                    if let Some(voxel) = chunk_map.map_data.get_mut(&chunk_key) {
                        // 1. 更新 chunk_map 数据
                        type SampleShape =
                            ConstShape3u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32, CHUNK_SIZE_U32>;
                        let index = SampleShape::linearize(pos) as usize;
                        if voxel[index].id == BasicStone::ID {
                            warn!("基岩无法破坏");
                            continue;
                        }
                        voxel[index] = voxel_type;
                        // 2. 更新 db 数据
                        let new_voxels_clone = voxel.clone();
                        let task =
                            pool.spawn(async move { (chunk_key.as_u8_array(), new_voxels_clone) });
                        db_save_task.tasks.push(task);
                        // 3. 通知 全体 更新数据
                        let message = bincode::serialize(&ChunkResult::ChunkUpdateOne {
                            chunk_key,
                            pos,
                            voxel_type,
                        })
                        .unwrap();
                        server.broadcast_message(ServerChannel::ChunkResult, message);
                        // 4. 判断 并更新codiller 存在的情况下才更新
                        send_codiller_task(
                            chunk_key,
                            &collider_manager,
                            &chunk_map,
                            &mut collider_update_tasks_manager,
                            &mut collider_tasks,
                        );
                        if pos[0] == 0 {
                            let mut new_chunk_key_i3 = chunk_key.0;
                            new_chunk_key_i3.x -= 1;
                            send_codiller_task(
                                ChunkKey(new_chunk_key_i3),
                                &collider_manager,
                                &chunk_map,
                                &mut collider_update_tasks_manager,
                                &mut collider_tasks,
                            );
                        }
                        if pos[0] == CHUNK_SIZE_U32 - 1 {
                            let mut new_chunk_key_i3 = chunk_key.0;
                            new_chunk_key_i3.x += 1;
                            send_codiller_task(
                                ChunkKey(new_chunk_key_i3),
                                &collider_manager,
                                &chunk_map,
                                &mut collider_update_tasks_manager,
                                &mut collider_tasks,
                            );
                        }
                        if pos[1] == 0 {
                            let mut new_chunk_key_i3 = chunk_key.0;
                            new_chunk_key_i3.y -= 1;
                            send_codiller_task(
                                ChunkKey(new_chunk_key_i3),
                                &collider_manager,
                                &chunk_map,
                                &mut collider_update_tasks_manager,
                                &mut collider_tasks,
                            );
                        }
                        if pos[1] == CHUNK_SIZE_U32 - 1 {
                            let mut new_chunk_key_i3 = chunk_key.0;
                            new_chunk_key_i3.y += 1;
                            send_codiller_task(
                                ChunkKey(new_chunk_key_i3),
                                &collider_manager,
                                &chunk_map,
                                &mut collider_update_tasks_manager,
                                &mut collider_tasks,
                            );
                        }
                        if pos[2] == 0 {
                            let mut new_chunk_key_i3 = chunk_key.0;
                            new_chunk_key_i3.z -= 1;
                            send_codiller_task(
                                ChunkKey(new_chunk_key_i3),
                                &collider_manager,
                                &chunk_map,
                                &mut collider_update_tasks_manager,
                                &mut collider_tasks,
                            );
                        }
                        if pos[2] == CHUNK_SIZE_U32 - 1 {
                            let mut new_chunk_key_i3 = chunk_key.0;
                            new_chunk_key_i3.z += 1;
                            send_codiller_task(
                                ChunkKey(new_chunk_key_i3),
                                &collider_manager,
                                &chunk_map,
                                &mut collider_update_tasks_manager,
                                &mut collider_tasks,
                            );
                        }
                    }
                }
            }
        }
    }
}

fn send_codiller_task(
    chunk_key: ChunkKey,
    collider_manager: &ColliderManager,
    chunk_map: &ChunkMap,
    collider_update_tasks_manager: &mut ColliderUpdateTasksManager,
    collider_tasks: &mut ColliderTasksManager,
) {
    let pool = AsyncComputeTaskPool::get();
    if let Some(&entity) = collider_manager.entities.get(&chunk_key) {
        let new_voxels_clone = chunk_map.get_neighbors(chunk_key);
        let task = pool.spawn(async move { (entity, chunk_key, new_voxels_clone) });
        collider_update_tasks_manager.tasks.push(task);
    } else if chunk_map.map_data.contains_key(&chunk_key) {
        let voxel_with_neighbor = chunk_map.get_neighbors(chunk_key);
        let task = pool.spawn(async move { (chunk_key, voxel_with_neighbor) });
        collider_tasks.tasks.push(task);
    }
}

pub fn send_message(mut tasks: ResMut<ChunkResultTasks>, mut server: ResMut<RenetServer>) {
    let l = tasks.tasks.len().min(16);
    for ele in tasks.tasks.drain(..l) {
        if let Some((client_id, message)) =
            futures_lite::future::block_on(futures_lite::future::poll_once(ele))
        {
            server.send_message(client_id, ServerChannel::ChunkResult, message);
        }
    }
}

pub struct ChunkDataPlugin;

impl Plugin for ChunkDataPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ChunkResultTasks { tasks: Vec::new() });
        app.add_systems(Update, (deal_chunk_query_system, send_message));
    }
}
