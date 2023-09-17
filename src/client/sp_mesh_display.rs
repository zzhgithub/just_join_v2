// 特殊类型的数据的更新

// 1. 生成 需要更新的数据 抛出任务
// 2. 处理任务 生成entity实体
// 3. 处理移动出去后的清理问题

use bevy::{
    prelude::{
        Assets, Color, Commands, Entity, OnExit, PbrBundle, Plugin, Res, ResMut, Resource,
        StandardMaterial, Transform, Update, Vec3,
    },
    tasks::{AsyncComputeTaskPool, Task},
    utils::{HashMap, HashSet},
};
use ndshape::{ConstShape, ConstShape3u32};

use crate::{
    common::ClipSpheres,
    tools::chunk_key_any_xyz_to_vec3,
    voxel_world::{
        chunk::{find_chunk_keys_array_by_sphere, generate_offset_array, ChunkKey},
        chunk_map::ChunkMap,
        voxel::{Voxel, VoxelDirection},
        voxel_mesh::{VoxelMeshStorge, VOXEL_MESH_MAP},
    },
    CHUNK_SIZE_U32, SP_MESH_DISTANCE,
};

use super::{
    mesh_display::{HitMeshType, TerrainMesh},
    state_manager::GameState,
};

#[derive(Debug, Clone, Resource)]
pub struct SpMeshManager {
    // chunk index
    pub entities: HashMap<ChunkKey, HashMap<u32, (Voxel, Entity)>>,
}

#[derive(Debug, Resource)]
pub struct SpMeshTasks {
    pub tasks: Vec<Task<(Voxel, ChunkKey, usize)>>,
}

pub struct SpMeshManagerPlugin;

impl Plugin for SpMeshManagerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(SpMeshManager {
            entities: HashMap::new(),
        });
        app.insert_resource(SpMeshTasks { tasks: Vec::new() });
        app.add_systems(
            Update,
            (sp_mesh_tasks_update, deal_sp_mesh_tasks, despawn_sp_mesh),
        );
        app.add_systems(OnExit(GameState::Game), unset_all);
    }
}

// 通过球体生成 要关注的
fn sp_mesh_tasks_update(
    mut commands: Commands,
    chunk_map: Res<ChunkMap>,
    mut sp_mesh_tasks: ResMut<SpMeshTasks>,
    clip_spheres: Res<ClipSpheres>,
    mut sp_mesh_manager: ResMut<SpMeshManager>,
) {
    let pool = AsyncComputeTaskPool::get();
    for chunk_key in find_chunk_keys_array_by_sphere(
        clip_spheres.new_sphere,
        generate_offset_array(SP_MESH_DISTANCE),
    ) {
        if let Some(voxels) = chunk_map.get(chunk_key.clone()) {
            let mut old_keys: HashSet<u32> = HashSet::new();
            // 获取特殊数据
            if let Some(index_voxels_entity_map) = sp_mesh_manager.entities.get_mut(&chunk_key) {
                for key in index_voxels_entity_map.keys() {
                    old_keys.insert(*key);
                }
            }
            for i in 0..voxels.len() {
                if VOXEL_MESH_MAP.contains_key(&voxels[i].id) {
                    if let Some(index_voxels_entity_map) =
                        sp_mesh_manager.entities.get_mut(&chunk_key)
                    {
                        // 原来有老数据 有要处理的数据
                        if let Some((v, entity)) = index_voxels_entity_map.get(&(i as u32)) {
                            // 如果方向发生了变化里面 马上改变方向
                            if v.direction != voxels[i].direction {
                                commands.entity(*entity).insert(get_tfr(
                                    chunk_key.clone(),
                                    i as u32,
                                    voxels[i].direction,
                                ));
                            }
                            // 否则什么也不处理
                            old_keys.remove(&(i as u32));
                        } else {
                            // 新创建的
                            println!("创建了新的sp");
                            let v = voxels[i].clone();
                            let task = pool.spawn(async move { (v, chunk_key.clone(), i) });
                            sp_mesh_tasks.tasks.push(task);
                        }
                    } else {
                        // 都是新数据
                        let v = voxels[i].clone();
                        let task = pool.spawn(async move { (v, chunk_key.clone(), i) });
                        println!("第一次创建的sp[{:?} on {}]", v, i);
                        sp_mesh_tasks.tasks.push(task);
                    }
                }
            }
            // 有一批要删除的数据
            for del_key in old_keys.iter() {
                if let Some(index_voxels_entity_map) = sp_mesh_manager.entities.get_mut(&chunk_key)
                {
                    if let Some((_, entity)) = index_voxels_entity_map.remove(del_key) {
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}

fn deal_sp_mesh_tasks(
    mut commands: Commands,
    mut sp_mesh_tasks: ResMut<SpMeshTasks>,
    mut sp_mesh_manager: ResMut<SpMeshManager>,
    // 配置数据
    voxel_mesh_storge: Res<VoxelMeshStorge>,
    mut stdmats: ResMut<Assets<StandardMaterial>>,
) {
    let l = sp_mesh_tasks.tasks.len().min(32);
    for ele in sp_mesh_tasks.tasks.drain(..l) {
        if let Some((v, chunk_key, index)) =
            futures_lite::future::block_on(futures_lite::future::poll_once(ele))
        {
            if let Some(meta) = voxel_mesh_storge.data.get(&v.id) {
                let entity = commands
                    .spawn(PbrBundle {
                        transform: get_tfr(chunk_key.clone(), index as u32, v.direction),
                        // FIXME:这里 暂时取第一个 后面要根据生成器取置换
                        mesh: meta.vox_list[0].clone(),
                        material: stdmats.add(Color::rgb(1., 1., 1.).into()),
                        ..Default::default()
                    })
                    // 修复这里的出现的中心的问题!
                    .insert(TerrainMesh(HitMeshType::Sp(get_pos(
                        chunk_key.clone(),
                        index as u32,
                    ))))
                    .id();
                if let Some(inner_map) = sp_mesh_manager.entities.get_mut(&chunk_key) {
                    inner_map.insert(index as u32, (v, entity));
                } else {
                    let mut map = HashMap::new();
                    map.insert(index as u32, (v, entity));
                    sp_mesh_manager.entities.insert(chunk_key, map);
                }
            }
        }
    }
}

// 这里要处理一下 mesh的销毁
fn despawn_sp_mesh(
    mut commands: Commands,
    mut sp_mesh_manager: ResMut<SpMeshManager>,
    clip_spheres: Res<ClipSpheres>,
) {
    let mut del_keys: HashSet<ChunkKey> = HashSet::new();
    for chunk_key in find_chunk_keys_array_by_sphere(
        clip_spheres.old_sphere,
        generate_offset_array(SP_MESH_DISTANCE),
    ) {
        del_keys.insert(chunk_key);
    }
    for chunk_key in find_chunk_keys_array_by_sphere(
        clip_spheres.new_sphere,
        generate_offset_array(SP_MESH_DISTANCE),
    ) {
        del_keys.remove(&chunk_key);
    }
    if del_keys.len() > 0 {
        for key in del_keys {
            if let Some(inner_map) = sp_mesh_manager.entities.remove(&key) {
                println!("要清理的数据{}", inner_map.len());
                for (_, (_, entity)) in inner_map {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

fn unset_all(
    mut commands: Commands,
    mut sp_mesh_tasks: ResMut<SpMeshTasks>,
    mut sp_mesh_manager: ResMut<SpMeshManager>,
) {
    // 这里释放所有的资源
    sp_mesh_tasks.tasks.clear();

    let keys: Vec<ChunkKey> = sp_mesh_manager.entities.keys().map(|a| a.clone()).collect();
    for chunk_key in keys.iter() {
        if let Some(inner_map) = sp_mesh_manager.entities.remove(chunk_key) {
            for (_, (_, entity)) in inner_map.into_iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

// 通过球体生成要删除的数据
pub type SampleShape = ConstShape3u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32, CHUNK_SIZE_U32>;

fn get_pos(chunk_key: ChunkKey, index: u32) -> Vec3 {
    let xyz = SampleShape::delinearize(index);
    chunk_key_any_xyz_to_vec3(chunk_key, xyz)
}

fn get_tfr(chunk_key: ChunkKey, index: u32, direction: VoxelDirection) -> Transform {
    Transform {
        translation: get_pos(chunk_key, index),
        rotation: direction.to_quat(),
        scale: Vec3::new(1. / 32., 1. / 32., 1. / 32.),
    }
}
