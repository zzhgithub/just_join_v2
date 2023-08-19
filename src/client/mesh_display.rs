use std::collections::HashSet;

use bevy::{
    prelude::{
        AlphaMode, AssetServer, Assets, Color, Commands, Entity, Handle, IntoSystemConfigs, Last,
        MaterialMeshBundle, MaterialPlugin, Mesh, Plugin, PreUpdate, Res, ResMut, Resource,
        StandardMaterial, Startup, Transform, Update,
    },
    tasks::{AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use bevy_renet::renet::RenetClient;

use crate::{
    common::ClipSpheres,
    server::{chunk_result::ChunkResult, server_channel::ServerChannel},
    tools::all_empty,
    voxel_world::{
        chunk::{
            find_chunk_keys_array_by_shpere_y_0, generate_offset_resoure, ChunkKey, NeighbourOffest,
        },
        chunk_map::ChunkMap,
        voxel::Voxel,
    },
    CHUNK_SIZE, MATERIAL_RON, VIEW_RADIUS,
};

use super::{
    chunk_query::ChunkQuery,
    client_channel::ClientChannel,
    voxels::{
        mesh::{gen_mesh, gen_mesh_water, pick_water},
        mesh_material::{BindlessMaterial, MaterialStorge},
        voxel_materail_config::MaterailConfiguration,
    },
};

#[derive(Debug, Clone, Resource, Default)]
pub struct MeshManager {
    pub mesh_storge: HashMap<ChunkKey, Handle<Mesh>>,
    pub water_mesh_storge: HashMap<ChunkKey, Handle<Mesh>>,
    pub entities: HashMap<ChunkKey, Entity>,
    pub water_entities: HashMap<ChunkKey, Entity>,
    pub fast_key: HashSet<ChunkKey>,
    pub data_status: HashMap<ChunkKey, bool>,
}

#[derive(Resource)]
pub struct MeshTasks {
    pub tasks: Vec<Task<(Vec<Voxel>, ChunkKey)>>,
}

#[derive(Resource)]
pub struct ChunkSyncTask {
    pub tasks: Vec<Task<(ChunkKey, Vec<Voxel>)>>,
}
pub struct ClientMeshPlugin;

impl Plugin for ClientMeshPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(MaterialPlugin::<BindlessMaterial>::default());
        app.insert_resource(ChunkMap::new());
        app.insert_resource(MeshManager::default());
        app.insert_resource(MeshTasks { tasks: Vec::new() });
        app.insert_resource(generate_offset_resoure(VIEW_RADIUS));
        app.insert_resource(ChunkSyncTask { tasks: Vec::new() });
        app.add_systems(Startup, setup);

        // mesh_加载和更新相关
        app.add_systems(
            PreUpdate,
            (gen_mesh_system, async_chunk_result).run_if(bevy_renet::transport::client_connected()),
        );
        app.add_systems(Update, (update_mesh_system, save_chunk_result));
        app.add_systems(Last, deleter_mesh_system);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    materials: ResMut<Assets<BindlessMaterial>>,
) {
    // 初始化数据
    let config = MaterailConfiguration::new()
        .read_file(String::from(MATERIAL_RON))
        .unwrap();
    commands.insert_resource(config.clone());
    commands.insert_resource(MaterialStorge::init_with_files(
        asset_server,
        materials,
        config.files.clone(),
    ));
}

pub fn gen_mesh_system(
    chunk_map: Res<ChunkMap>,
    mut mesh_manager: ResMut<MeshManager>,
    clip_spheres: Res<ClipSpheres>,
    neighbour_offest: Res<NeighbourOffest>,
    mut mesh_task: ResMut<MeshTasks>,
    mut client: ResMut<RenetClient>,
) {
    let pool = AsyncComputeTaskPool::get();
    for key in
        find_chunk_keys_array_by_shpere_y_0(clip_spheres.new_sphere, neighbour_offest.0.clone())
            .drain(..)
    {
        if !mesh_manager.entities.contains_key(&key) && !mesh_manager.fast_key.contains(&key) {
            // FIXME: 这要给数据加上 一个有效时间放置server端丢命令
            if let Some(_state) = mesh_manager.data_status.get(&key) {
                if chunk_map.chunk_for_mesh_ready(key) {
                    mesh_manager.fast_key.insert(key);
                    mesh_manager.data_status.insert(key, true);
                    let volexs: Vec<Voxel> = chunk_map.get_with_neighbor_full_y(key);
                    let task = pool.spawn(async move { (volexs.clone(), key.clone()) });
                    mesh_task.tasks.push(task);
                }
            } else {
                let message = bincode::serialize(&ChunkQuery::GetFullY(key)).unwrap();
                client.send_message(ClientChannel::ChunkQuery, message);
                mesh_manager.data_status.insert(key, false);
            }
        }
    }
}

pub fn async_chunk_result(
    mut client: ResMut<RenetClient>,
    mut chunk_sync_task: ResMut<ChunkSyncTask>,
) {
    let pool = AsyncComputeTaskPool::get();
    while let Some(message) = client.receive_message(ServerChannel::ChunkResult) {
        let chunk_result: ChunkResult = bincode::deserialize(&message).unwrap();
        match chunk_result {
            ChunkResult::ChunkData { key, data } => {
                // println!("{:?}",data);
                let task = pool.spawn(async move { (key, data) });
                chunk_sync_task.tasks.push(task);
            }
        }
    }
}

pub fn save_chunk_result(
    mut chunk_sync_task: ResMut<ChunkSyncTask>,
    mut chunk_map: ResMut<ChunkMap>,
) {
    let l = chunk_sync_task.tasks.len().min(16);
    for ele in chunk_sync_task.tasks.drain(..l) {
        match futures_lite::future::block_on(futures_lite::future::poll_once(ele)) {
            Some((chunk_key, data)) => {
                if all_empty(data.clone()) {
                } else {
                    println!("{:?}", chunk_key);
                }
                chunk_map.write_chunk(chunk_key, data);
            }
            None => {}
        }
    }
}

pub fn update_mesh_system(
    mut commands: Commands,
    mut mesh_manager: ResMut<MeshManager>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut mesh_task: ResMut<MeshTasks>,
    materials: Res<MaterialStorge>,
    material_config: Res<MaterailConfiguration>,
    mut materials_assets: ResMut<Assets<StandardMaterial>>,
) {
    let l: usize = mesh_task.tasks.len().min(3);
    for ele in mesh_task.tasks.drain(..l) {
        match futures_lite::future::block_on(futures_lite::future::poll_once(ele)) {
            Some((voxels, chunk_key)) => {
                if mesh_manager.entities.contains_key(&chunk_key) {
                    return;
                } else {
                    match gen_mesh(voxels.to_owned(), material_config.clone()) {
                        Some(render_mesh) => {
                            let mesh_handle = mesh_assets.add(render_mesh);
                            mesh_manager
                                .mesh_storge
                                .insert(chunk_key, mesh_handle.clone());
                            mesh_manager.entities.insert(
                                chunk_key,
                                commands
                                    .spawn(MaterialMeshBundle {
                                        transform: Transform::from_xyz(
                                            (chunk_key.0.x * CHUNK_SIZE) as f32
                                                - CHUNK_SIZE as f32 / 2.0
                                                - 1.0,
                                            -128.0 + CHUNK_SIZE as f32 / 2.0,
                                            (chunk_key.0.z * CHUNK_SIZE) as f32
                                                - CHUNK_SIZE as f32 / 2.0
                                                - 1.0,
                                        ),
                                        mesh: mesh_handle.clone(),
                                        material: materials.0.clone(),
                                        ..Default::default()
                                    })
                                    .id(),
                            );
                        }
                        None => {
                            println!("All None");
                        }
                    };
                    match gen_mesh_water(pick_water(voxels.clone()), material_config.clone()) {
                        Some(water_mesh) => {
                            let water_mesh_handle = mesh_assets.add(water_mesh);
                            mesh_manager
                                .water_mesh_storge
                                .insert(chunk_key, water_mesh_handle.clone());
                            mesh_manager.water_entities.insert(
                                chunk_key,
                                commands
                                    .spawn(MaterialMeshBundle {
                                        transform: Transform::from_xyz(
                                            (chunk_key.0.x * CHUNK_SIZE) as f32
                                                - CHUNK_SIZE as f32 / 2.0
                                                - 1.0,
                                            -128.0 + CHUNK_SIZE as f32 / 2.0,
                                            (chunk_key.0.z * CHUNK_SIZE) as f32
                                                - CHUNK_SIZE as f32 / 2.0
                                                - 1.0,
                                        ),
                                        mesh: water_mesh_handle,
                                        material: materials_assets.add(StandardMaterial {
                                            base_color: Color::rgba(
                                                10. / 255.,
                                                18. / 255.,
                                                246. / 255.,
                                                0.6,
                                            ),
                                            alpha_mode: AlphaMode::Blend,
                                            ..Default::default()
                                        }),
                                        ..Default::default()
                                    })
                                    .id(),
                            );
                        }
                        None => {}
                    }
                }
            }
            None => {}
        }
    }
}

pub fn deleter_mesh_system(
    mut commands: Commands,
    mut mesh_manager: ResMut<MeshManager>,
    neighbour_offest: Res<NeighbourOffest>,
    clip_spheres: Res<ClipSpheres>,
) {
    let mut chunks_to_remove = HashSet::new();
    for key in
        find_chunk_keys_array_by_shpere_y_0(clip_spheres.old_sphere, neighbour_offest.0.clone())
            .drain(..)
    {
        chunks_to_remove.insert(key);
    }

    for key in
        find_chunk_keys_array_by_shpere_y_0(clip_spheres.new_sphere, neighbour_offest.0.clone())
            .drain(..)
    {
        chunks_to_remove.remove(&key);
    }

    for chunk_key in chunks_to_remove.into_iter() {
        if let Some(entity) = mesh_manager.entities.remove(&chunk_key) {
            mesh_manager.fast_key.remove(&chunk_key);
            commands.entity(entity).despawn();
        }
        if let Some(entity) = mesh_manager.water_entities.remove(&chunk_key) {
            mesh_manager.fast_key.remove(&chunk_key);
            commands.entity(entity).despawn();
        }
    }
}
