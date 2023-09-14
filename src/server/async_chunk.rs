use bevy::{
    prelude::{warn, EventWriter, Plugin, Query, Res, ResMut, Resource, Update},
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_renet::renet::RenetServer;
use ndshape::{ConstShape, ConstShape3u32};

use crate::{
    client::message_def::{chunk_query::ChunkQuery, ClientChannel},
    server::{message_def::ServerChannel, object_filing::put_object::put_object},
    staff::StaffInfoStroge,
    voxel_world::{
        biomes::OtherTreeTasksMap,
        chunk::ChunkKey,
        chunk_map::ChunkMap,
        compress::compress,
        map_database::{DbSaveTasks, MapDataBase},
        player_state::PlayerOnTimeState,
        voxel::{BasicStone, Voxel, VoxelMaterial},
    },
    CHUNK_SIZE, CHUNK_SIZE_U32,
};

use super::{
    message_def::chunk_result::ChunkResult,
    object_filing::ObjectFillEvent,
    player::ServerLobby,
    terrain_physics::{ColliderManager, ColliderTasksManager, ColliderUpdateTasksManager},
};

#[derive(Debug, Resource)]
pub struct ChunkResultTasks {
    pub tasks: Vec<Task<(u64, Vec<u8>)>>,
}

#[allow(clippy::too_many_arguments)]
pub fn deal_chunk_query_system(
    mut server: ResMut<RenetServer>,
    mut chunk_map: ResMut<ChunkMap>,
    mut db_save_task: ResMut<DbSaveTasks>,
    mut db: ResMut<MapDataBase>,
    mut tasks: ResMut<ChunkResultTasks>,
    collider_manager: Res<ColliderManager>,
    mut collider_update_tasks_manager: ResMut<ColliderUpdateTasksManager>,
    mut collider_tasks: ResMut<ColliderTasksManager>,
    mut fill_event: EventWriter<ObjectFillEvent>,
    staff_info_stroge: Res<StaffInfoStroge>,
    // 获取玩家当前状态 和处理
    mut query_state: Query<&mut PlayerOnTimeState>,
    server_lobby: Res<ServerLobby>,
    mut other_tree_tasks_map: ResMut<OtherTreeTasksMap>,
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
                            voxels = db.find_by_chunk_key(
                                new_key,
                                db_save_task.as_mut(),
                                other_tree_tasks_map.as_mut(),
                            );
                        }
                        let (buffer, tree) = compress(voxels.clone());
                        let message = if buffer.len() == 0 {
                            bincode::serialize(&ChunkResult::ChunkSame((new_key, voxels[0])))
                                .unwrap()
                        } else {
                            bincode::serialize(&ChunkResult::ChunkData {
                                key: new_key,
                                data: (buffer, tree),
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
                    center,
                    active_index,
                } => {
                    if let Some(voxel) = chunk_map.map_data.get_mut(&chunk_key) {
                        // 1. 更新 chunk_map 数据
                        type SampleShape =
                            ConstShape3u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32, CHUNK_SIZE_U32>;
                        let index = SampleShape::linearize(pos) as usize;
                        // 老的体素位置
                        let old_voxel = voxel[index].clone();
                        if voxel[index].id == BasicStone::ID {
                            warn!("基岩无法破坏");
                            continue;
                        }
                        if old_voxel.id != Voxel::EMPTY.id
                            && voxel_type.id != Voxel::EMPTY.id
                            && active_index != None
                        {
                            warn!("放置错误");
                            continue;
                        }
                        // 判断是否可以影响到数据 只有放置时才处理！
                        if voxel_type.id != Voxel::EMPTY.id {
                            if let Some(staff) =
                                staff_info_stroge.voxel_to_staff(voxel_type.clone())
                            {
                                if let Some(index) = active_index {
                                    if put_object(
                                        client_id,
                                        &server_lobby,
                                        &mut query_state,
                                        index,
                                        staff.id,
                                        &mut server,
                                    ) {
                                        // 发送成功
                                    } else {
                                        warn!("{}|无法从toolbar获取", client_id);
                                        continue;
                                    }
                                } else {
                                    warn!("{}|角色没有当前生成的index", client_id);
                                    if old_voxel.direction == voxel_type.direction {
                                        continue;
                                    } else {
                                        println!("转动方向");
                                    }
                                }
                            } else {
                                warn!("{}|{}没有找到资源对应关系", client_id, voxel_type.id);
                                continue;
                            }
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
                        // FIXME: 这里要考虑把代码格式简化 一下
                        // 发送物体被打下来的消息 old_voxel  chunk_key, pos, 还原物体的位置!
                        if old_voxel.id != Voxel::EMPTY.id && voxel_type.id == Voxel::EMPTY.id {
                            // 物体时被打下来了 这里通过配置掉落
                            if let Some(staff_list) =
                                staff_info_stroge.voxel_to_staff_list(old_voxel)
                            {
                                for staff in staff_list.into_iter() {
                                    fill_event.send(ObjectFillEvent {
                                        chunk_key,
                                        xyz: pos,
                                        center,
                                        staff: staff,
                                    });
                                }
                            }
                        }
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
            if client_id == 0 {
                server.broadcast_message(ServerChannel::ChunkResult, message);
            } else {
                server.send_message(client_id, ServerChannel::ChunkResult, message);
            }
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
