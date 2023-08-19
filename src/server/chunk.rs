use bevy::prelude::{Last, Plugin, Res, ResMut, Update};

use crate::{
    common::ServerClipSpheres,
    voxel_world::{
        chunk::{
            find_chunk_keys_by_shpere_to_full_height, generate_offset_resoure, NeighbourOffest,
        },
        chunk_map::ChunkMap,
        map_database::{save_db_task_system, DbSaveTasks, MapDataBase},
    },
    VIEW_RADIUS, WORD_PATH,
};

/**
 * 服务端生成 chunk数据
 */
pub fn server_chunk_generate_system(
    mut chunk_map: ResMut<ChunkMap>,
    neighbour_offest: Res<NeighbourOffest>,
    server_clip_spheres: Res<ServerClipSpheres>,
    mut db: ResMut<MapDataBase>,
    mut db_save_tasks: ResMut<DbSaveTasks>,
) {
    for (_client_id, clip_spheres) in server_clip_spheres.clip_spheres.iter() {
        // 通过球体计算 chunkey
        find_chunk_keys_by_shpere_to_full_height(
            clip_spheres.new_sphere,
            neighbour_offest.0.clone(),
            |key| {
                // 这里要判断一下获取的方法
                // chunk_map.gen_chunk_data(key);
                if !chunk_map.map_data.contains_key(&key) {
                    //  这里可以判断一下是否是 已经加载的数据
                    let data = db.find_by_chunk_key(key, db_save_tasks.as_mut());
                    chunk_map.write_chunk(key, data);
                }
            },
        );
    }
}

pub struct ServerChunkPlugin;

impl Plugin for ServerChunkPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // init MapData
        app.insert_resource(MapDataBase::new(WORD_PATH));
        app.insert_resource(generate_offset_resoure(VIEW_RADIUS));
        app.insert_resource(ChunkMap::new());
        app.insert_resource(DbSaveTasks { tasks: Vec::new() });

        app.add_systems(Update, server_chunk_generate_system);
        app.add_systems(Last, save_db_task_system);
    }
}
