use bevy::{
    prelude::{Plugin, Res, ResMut, Resource, Update},
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_renet::renet::RenetServer;

use crate::{
    client::{chunk_query::ChunkQuery, client_channel::ClientChannel},
    voxel_world::{
        chunk_map::ChunkMap,
        map_database::{DbSaveTasks, MapDataBase},
    },
    CHUNK_SIZE,
};

use super::{chunk_result::ChunkResult, server_channel::ServerChannel};

#[derive(Debug, Resource)]
pub struct ChunkResultTasks {
    pub tasks: Vec<Task<(u64, Vec<u8>)>>,
}

pub fn deal_chunk_query_system(
    mut server: ResMut<RenetServer>,
    chunk_map: Res<ChunkMap>,
    mut db_save_task: ResMut<DbSaveTasks>,
    mut db: ResMut<MapDataBase>,
    mut tasks: ResMut<ChunkResultTasks>,
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
                        let mut new_key = chunk_key.clone();
                        new_key.0.y = y_offset;
                        let message: Vec<u8>;
                        if let Some(data) = chunk_map.map_data.get(&new_key) {
                            message = bincode::serialize(&ChunkResult::ChunkData {
                                key: new_key.clone(),
                                data: data.clone(),
                            })
                            .unwrap();
                        } else {
                            let data = db.find_by_chunk_key(new_key, db_save_task.as_mut());
                            message = bincode::serialize(&ChunkResult::ChunkData {
                                key: new_key.clone(),
                                data: data,
                            })
                            .unwrap();
                        }
                        let task = pool.spawn(async move { (client_id, message) });
                        tasks.tasks.push(task);
                    }
                }
            }
        }
    }
}

pub fn send_message(mut tasks: ResMut<ChunkResultTasks>, mut server: ResMut<RenetServer>) {
    let l = tasks.tasks.len().min(16);
    for ele in tasks.tasks.drain(..l) {
        match futures_lite::future::block_on(futures_lite::future::poll_once(ele)) {
            Some((client_id, message)) => {
                server.send_message(client_id, ServerChannel::ChunkResult, message);
            }
            None => {}
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
