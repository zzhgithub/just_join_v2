use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use bevy::{
    prelude::{
        AlphaMode, AssetServer, Assets, Color, Commands, Component, Entity, Handle,
        IntoSystemConfigs, Last, MaterialMeshBundle, MaterialPlugin, Mesh, Plugin, PreUpdate, Res,
        ResMut, Resource, StandardMaterial, Startup, Transform, Update,
    },
    tasks::{AsyncComputeTaskPool, Task},
    time::{Time, Timer, TimerMode},
    utils::HashMap,
};
use bevy_mod_raycast::RaycastMesh;
use bevy_renet::renet::RenetClient;
use ndshape::{ConstShape, ConstShape3u32};

use crate::{
    common::ClipSpheres,
    server::{chunk_result::ChunkResult, server_channel::ServerChannel},
    tools::get_empty_chunk,
    voxel_world::{
        chunk::{
            find_chunk_keys_array_by_shpere_y_0, generate_offset_resoure,
            generate_offset_resoure_min_1, ChunkKey, NeighbourOffest,
        },
        chunk_map::ChunkMap,
        voxel::Voxel,
    },
    CHUNK_SIZE, CHUNK_SIZE_U32, MATERIAL_RON, VIEW_RADIUS,
};

use super::{
    chunk_query::ChunkQuery,
    client_channel::ClientChannel,
    ray_cast::MyRaycastSet,
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
    pub data_status: HashMap<ChunkKey, (bool, Instant)>,
}

#[derive(Resource)]
pub struct MeshTasks {
    pub tasks: Vec<Task<(Vec<Voxel>, ChunkKey)>>,
}

#[derive(Resource)]
pub struct ChunkSyncTask {
    pub tasks: Vec<Task<(ChunkKey, Vec<Voxel>)>>,
}

#[derive(Resource)]
pub struct ChunkUpdateTask {
    pub tasks: Vec<Task<ChunkKey>>,
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
        app.insert_resource(ChunkUpdateTask { tasks: Vec::new() });
        app.insert_resource(CycleCheckTimer(Timer::new(
            bevy::utils::Duration::from_millis(1000 * 2),
            TimerMode::Repeating,
        )));
        app.add_systems(Startup, setup);

        // mesh_加载和更新相关
        app.add_systems(
            PreUpdate,
            (gen_mesh_system, async_chunk_result, cycle_check_mesh)
                .run_if(bevy_renet::transport::client_connected()),
        );
        app.add_systems(
            Update,
            (update_mesh_system, save_chunk_result, update_chunk_mesh),
        );
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
        config.files,
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
                    mesh_manager.data_status.insert(key, (true, Instant::now()));
                    let volexs: Vec<Voxel> = chunk_map.get_with_neighbor_full_y(key);
                    let task = pool.spawn(async move { (volexs.clone(), key) });
                    mesh_task.tasks.push(task);
                }
            } else if !chunk_map.chunk_for_mesh_ready(key) {
                let message = bincode::serialize(&ChunkQuery::GetFullY(key)).unwrap();
                client.send_message(ClientChannel::ChunkQuery, message);
                mesh_manager
                    .data_status
                    .insert(key, (false, Instant::now()));
            } else {
                mesh_manager
                    .data_status
                    .insert(key, (false, Instant::now()));
            }
        }
    }
}

pub fn async_chunk_result(
    mesh_manager: Res<MeshManager>,
    mut client: ResMut<RenetClient>,
    mut chunk_sync_task: ResMut<ChunkSyncTask>,
    mut chunk_map: ResMut<ChunkMap>,
    mut chunk_update_task: ResMut<ChunkUpdateTask>,
) {
    let pool = AsyncComputeTaskPool::get();
    let mut key_set: HashSet<(usize, ChunkKey)> = HashSet::new();
    while let Some(message) = client.receive_message(ServerChannel::ChunkResult) {
        let chunk_result: ChunkResult = bincode::deserialize(&message).unwrap();
        match chunk_result {
            ChunkResult::ChunkData { key, data } => {
                let task = pool.spawn(async move { (key, data) });
                chunk_sync_task.tasks.push(task);
            }
            ChunkResult::ChunkEmpty(key) => {
                let task = pool.spawn(async move { (key, get_empty_chunk()) });
                chunk_sync_task.tasks.push(task);
            }
            ChunkResult::ChunkUpdateOne {
                chunk_key,
                pos,
                voxel_type,
            } => {
                // 1. 判断 更新 chunkmap的数据
                if let Some(voxel) = chunk_map.map_data.get_mut(&chunk_key) {
                    type SampleShape =
                        ConstShape3u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32, CHUNK_SIZE_U32>;
                    let index = SampleShape::linearize(pos) as usize;
                    voxel[index] = voxel_type;
                    let mut clone_chunk_key = chunk_key;
                    clone_chunk_key.0.y = 0;
                    key_set.insert((1, clone_chunk_key));
                    // 2. 刷新mesh的task 注意是刷新的task
                    if pos[0] == 0 {
                        let mut new_chunk_key_i3 = clone_chunk_key.0;
                        new_chunk_key_i3.x -= 1;
                        key_set.insert((0, ChunkKey(new_chunk_key_i3)));
                    }
                    if pos[0] == CHUNK_SIZE_U32 - 1 {
                        let mut new_chunk_key_i3 = clone_chunk_key.0;
                        new_chunk_key_i3.x += 1;
                        key_set.insert((0, ChunkKey(new_chunk_key_i3)));
                    }
                    if pos[2] == 0 {
                        let mut new_chunk_key_i3 = clone_chunk_key.0;
                        new_chunk_key_i3.z -= 1;
                        key_set.insert((0, ChunkKey(new_chunk_key_i3)));
                    }
                    if pos[2] == CHUNK_SIZE_U32 - 1 {
                        let mut new_chunk_key_i3 = clone_chunk_key.0;
                        new_chunk_key_i3.z += 1;
                        key_set.insert((0, ChunkKey(new_chunk_key_i3)));
                    }
                }
            }
        }
    }
    // 这里解决处理顺序
    let mut sorted_vec: Vec<(usize, ChunkKey)> = key_set.into_iter().collect();
    sorted_vec.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().reverse());

    for (_, key) in sorted_vec.iter() {
        let chunk_key: ChunkKey = *key;
        if mesh_manager.entities.get(key).is_some() {
            let task = pool.spawn(async move { chunk_key });
            chunk_update_task.tasks.push(task);
        }
    }
}

pub fn update_chunk_mesh(
    mut commands: Commands,
    mut chunk_update_task: ResMut<ChunkUpdateTask>,
    material_config: Res<MaterailConfiguration>,
    chunk_map: Res<ChunkMap>,
    mut mesh_manager: ResMut<MeshManager>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
) {
    let l = chunk_update_task.tasks.len().min(16);
    for ele in chunk_update_task.tasks.drain(..l) {
        if let Some(chunk_key) =
            futures_lite::future::block_on(futures_lite::future::poll_once(ele))
        {
            update_mesh(
                &mut commands,
                chunk_map.as_ref(),
                chunk_key,
                material_config.clone(),
                mesh_manager.as_mut(),
                mesh_assets.as_mut(),
            )
        }
    }
}

pub fn update_mesh(
    commands: &mut Commands,
    chunk_map: &ChunkMap,
    chunk_key_y0: ChunkKey,
    material_config: MaterailConfiguration,
    mesh_manager: &mut MeshManager,
    mesh_assets: &mut Assets<Mesh>,
) {
    let volexs: Vec<Voxel> = chunk_map.get_with_neighbor_full_y(chunk_key_y0);
    match gen_mesh(volexs.to_owned(), material_config.clone()) {
        Some(render_mesh) => {
            let mesh_handle = mesh_manager.mesh_storge.get(&chunk_key_y0).unwrap();
            if let Some(mesh) = mesh_assets.get_mut(mesh_handle) {
                *mesh = render_mesh;
            }
            // 没有生成mesh就不管反正后面要生成
        }
        None => {
            mesh_manager.mesh_storge.remove(&chunk_key_y0);
            if let Some(entity) = mesh_manager.entities.remove(&chunk_key_y0) {
                commands.entity(entity).despawn();
            }
        }
    };
    match gen_mesh_water(pick_water(volexs), material_config) {
        Some(water_mesh) => {
            let mesh_handle = mesh_manager.water_mesh_storge.get(&chunk_key_y0).unwrap();
            if let Some(mesh) = mesh_assets.get_mut(mesh_handle) {
                *mesh = water_mesh;
            }
        }
        None => {
            mesh_manager.water_mesh_storge.remove(&chunk_key_y0);
            if let Some(entity) = mesh_manager.water_entities.remove(&chunk_key_y0) {
                commands.entity(entity).despawn();
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
        if let Some((chunk_key, data)) =
            futures_lite::future::block_on(futures_lite::future::poll_once(ele))
        {
            chunk_map.write_chunk(chunk_key, data);
        }
    }
}

#[derive(Debug, Component)]
pub struct TerrainMesh;

#[derive(Debug, Component)]
pub struct WaterMesh;

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
        if let Some((voxels, chunk_key)) =
            futures_lite::future::block_on(futures_lite::future::poll_once(ele))
        {
            if mesh_manager.entities.contains_key(&chunk_key) {
                return;
            } else {
                if let Some(render_mesh) = gen_mesh(voxels.to_owned(), material_config.clone()) {
                    let mesh_handle = mesh_assets.add(render_mesh);
                    mesh_manager
                        .mesh_storge
                        .insert(chunk_key, mesh_handle.clone());
                    mesh_manager.entities.insert(
                        chunk_key,
                        commands
                            .spawn((
                                MaterialMeshBundle {
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
                                },
                                TerrainMesh,
                                RaycastMesh::<MyRaycastSet>::default(), // Make this mesh ray cast-able
                            ))
                            .id(),
                    );
                };
                if let Some(water_mesh) =
                    gen_mesh_water(pick_water(voxels.clone()), material_config.clone())
                {
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
                            .insert(WaterMesh)
                            .id(),
                    );
                }
            }
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
        mesh_manager.data_status.remove(&chunk_key);
    }
}

// 定期检查丢包问题

#[derive(Resource)]
pub struct CycleCheckTimer(Timer);

fn cycle_check_mesh(
    mut timer: ResMut<CycleCheckTimer>,
    time: Res<Time>,
    mut mesh_manager: ResMut<MeshManager>,
    mut client: ResMut<RenetClient>,
    clip_spheres: Res<ClipSpheres>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        let need_keys: HashSet<ChunkKey> = find_chunk_keys_array_by_shpere_y_0(
            clip_spheres.new_sphere,
            generate_offset_resoure_min_1(VIEW_RADIUS).0,
        )
        .iter()
        .copied()
        .collect();

        for (key, (state, instant)) in mesh_manager.data_status.clone().iter() {
            let now: Instant = Instant::now();
            let duration: Duration = now - *instant;
            // 每2s检查一下五秒内没有加载好的数据
            if !state && duration.as_millis() > 5 * 1000 && need_keys.contains(key) {
                println!("超时重新请求chunkkey{:?}", key);
                let message = bincode::serialize(&ChunkQuery::GetFullY(*key)).unwrap();
                // todo 对边缘数据不处理！
                client.send_message(ClientChannel::ChunkQuery, message);
            }
            if duration.as_millis() > 5 * 1000
                && !mesh_manager.entities.contains_key(key)
                && mesh_manager.fast_key.contains(key)
            {
                println!("数据修复2");
                mesh_manager.fast_key.remove(key);
            }
        }
    }
}
