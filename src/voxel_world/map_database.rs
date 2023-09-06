// 使用数据数据

use bevy::{
    prelude::{ResMut, Resource},
    tasks::{AsyncComputeTaskPool, Task},
};
use ndshape::{ConstShape, ConstShape3u32};
use sled::Db;

use crate::{voxel_world::map_generator::gen_chunk_data_by_seed, CHUNK_SIZE_U32, CLIENT_MAP_GEN};

use super::{chunk::ChunkKey, voxel::Voxel};

#[derive(Resource)]
pub struct MapDataBase {
    pub db: Db,
}

impl MapDataBase {
    pub fn new(path: &str) -> Self {
        let db = sled::open(path).unwrap();
        Self { db }
    }

    // 通过chunkKey 查找体素数据
    pub fn find_by_chunk_key(
        &mut self,
        chunk_key: ChunkKey,
        db_tasks: &mut DbSaveTasks,
    ) -> Vec<Voxel> {
        let pool = AsyncComputeTaskPool::get();
        let mut voxels = Vec::new();
        type SampleShape = ConstShape3u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32, CHUNK_SIZE_U32>;
        for _ in 0..SampleShape::SIZE {
            voxels.push(Voxel::EMPTY);
        }
        let key = chunk_key.as_u8_array();
        match self.db.get(key) {
            Ok(rs) => match if CLIENT_MAP_GEN { None } else { rs } {
                Some(data) => bincode::deserialize(&data).unwrap(),
                // 这里在没有获取到的情况下使用算法的值
                None => {
                    let new_voxels = gen_chunk_data_by_seed(1512354854, chunk_key);
                    let new_voxels_clone = new_voxels.clone();
                    let task = pool.spawn(async move { (key, new_voxels_clone) });
                    db_tasks.tasks.push(task);
                    new_voxels
                }
            },
            Err(e) => {
                println!("wrong, to get Map {:?}", e);
                voxels
            }
        }
    }
}

#[derive(Debug, Resource)]
pub struct DbSaveTasks {
    pub tasks: Vec<Task<([u8; 8], Vec<Voxel>)>>,
}

pub fn save_db_task_system(mut db_save_task: ResMut<DbSaveTasks>, db: ResMut<MapDataBase>) {
    // 一次最多处理6个
    let len = db_save_task.tasks.len().min(6);
    for ele in db_save_task.tasks.drain(..len) {
        if let Some((key, data)) =
            futures_lite::future::block_on(futures_lite::future::poll_once(ele))
        {
            match db.db.insert(key, bincode::serialize(&data).unwrap()) {
                Ok(_) => {
                    // println!("数据保存成功");
                }
                Err(err) => {
                    println!("数据保存问题{:?}", err);
                }
            }
        }
    }
}
