// 使用数据数据

use bevy::prelude::Resource;
use ndshape::{ConstShape, ConstShape3u32};
use sled::Db;

use crate::{voxel_world::map_generator::gen_chunk_data_by_seed, CHUNK_SIZE_U32};

use super::{chunk::ChunkKey, voxel::Voxel};

#[derive(Resource)]
pub struct MapDataBase {
    pub db: Db,
}

impl MapDataBase {
    pub fn new(path: &str) -> Self {
        let db = sled::open(path).unwrap();
        Self { db: db }
    }

    // 通过chunkKey 查找体素数据
    pub fn find_by_chunk_key(&self, chunk_key: ChunkKey) -> Vec<Voxel> {
        let mut voxels = Vec::new();
        type SampleShape = ConstShape3u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32, CHUNK_SIZE_U32>;
        for _ in 0..SampleShape::SIZE {
            voxels.push(Voxel::EMPTY);
        }
        let key = chunk_key.as_u8_array();
        return match self.db.get(key) {
            Ok(rs) => match rs {
                Some(data) => bincode::deserialize(&data).unwrap(),
                // 这里在没有获取到的情况下使用算法的值
                None => {
                    let new_voxels = gen_chunk_data_by_seed(1512354854, chunk_key);
                    match self
                        .db
                        .insert(key, bincode::serialize(&new_voxels).unwrap())
                    {
                        Ok(_) => {
                            println!("数据保存成功");
                        }
                        Err(err) => {
                            println!("数据保存问题{:?}", err);
                        }
                    }
                    new_voxels
                }
            },
            Err(e) => {
                println!("wrong, to get Map {:?}", e);
                voxels
            }
        };
    }
}
