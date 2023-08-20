use std::collections::HashSet;

use bevy::{
    prelude::{
        Commands, Component, Entity, GlobalTransform, IntoSystemConfigs, Last, Plugin, PreUpdate,
        Query, Res, ResMut, Resource, SystemSet, Transform, Vec3,
    },
    tasks::{AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use bevy_rapier3d::prelude::{Collider, RigidBody};
use block_mesh::{greedy_quads, GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG};
use ndshape::{ConstShape, ConstShape3u32};

use crate::{
    common::ServerClipSpheres,
    voxel_world::{
        chunk::{find_chunk_keys_array_by_shpere, generate_offset_array, ChunkKey},
        chunk_map::ChunkMap,
        voxel::Voxel,
    },
    CHUNK_SIZE, CHUNK_SIZE_ADD_2_U32, PY_DISTANCE,
};

#[derive(Debug, Component)]
pub struct TerrainPhysics;

// 管理碰撞体的组件
#[derive(Debug, Resource)]
pub struct ColliderManager {
    pub entities: HashMap<ChunkKey, Entity>,
}

#[derive(Debug, Resource, Default)]
pub struct ColliderTasksManager {
    pub tasks: Vec<Task<(ChunkKey, Vec<Voxel>)>>,
}

pub fn server_update_collider_task_system(
    chunk_map: Res<ChunkMap>,
    collider_manager: Res<ColliderManager>,
    mut collider_tasks: ResMut<ColliderTasksManager>,
    server_clip_spheres: Res<ServerClipSpheres>,
) {
    let pool = AsyncComputeTaskPool::get();
    for (_client_id, clip_spheres) in server_clip_spheres.clip_spheres.iter() {
        for chunk_key in find_chunk_keys_array_by_shpere(
            clip_spheres.new_sphere,
            generate_offset_array(PY_DISTANCE),
        )
        .drain(..)
        {
            // 在没有创建过的情况下才进行
            // FIXME: 这里在一开始的时候似乎就可以 过滤一些的
            if !collider_manager.entities.contains_key(&chunk_key)
                && chunk_map.map_data.contains_key(&chunk_key)
            {
                let voxel_with_neighbor = chunk_map.get_neighbors(chunk_key);
                let task = pool.spawn(async move { (chunk_key, voxel_with_neighbor) });
                collider_tasks.tasks.push(task);
            }
        }
    }
}

// 生成碰撞体
pub fn spawn_collider(
    mut collider_tasks: ResMut<ColliderTasksManager>,
    mut collider_manager: ResMut<ColliderManager>,
    mut commands: Commands,
) {
    let len = collider_tasks.tasks.len().min(256);
    for ele in collider_tasks.tasks.drain(..len) {
        match futures_lite::future::block_on(futures_lite::future::poll_once(ele)) {
            Some((chunk_key, voxel_data)) => {
                if !collider_manager.entities.contains_key(&chunk_key) {
                    if let Some(collider) = gen_collider(voxel_data) {
                        let entity = commands
                            .spawn((
                                TerrainPhysics,
                                Transform::from_xyz(
                                    (chunk_key.0.x * CHUNK_SIZE) as f32
                                        - CHUNK_SIZE as f32 / 2.0
                                        - 1.0,
                                    (chunk_key.0.y * CHUNK_SIZE) as f32
                                        - CHUNK_SIZE as f32 / 2.0
                                        - 1.0,
                                    (chunk_key.0.z * CHUNK_SIZE) as f32
                                        - CHUNK_SIZE as f32 / 2.0
                                        - 1.0,
                                ),
                                GlobalTransform::default(),
                            ))
                            .insert(RigidBody::Fixed)
                            .insert(collider)
                            .id();
                        collider_manager.entities.insert(chunk_key, entity);
                    }
                }
            }
            None => {}
        }
    }
}

pub fn despawn_collider(
    server_clip_spheres: Res<ServerClipSpheres>,
    mut collider_manager: ResMut<ColliderManager>,
    mut commands: Commands,
) {
    let neighbour_offest = generate_offset_array(PY_DISTANCE);
    let mut chunks_to_remove = HashSet::new();
    for (_client_id, clip_spheres) in server_clip_spheres.clip_spheres.iter() {
        for key in
            find_chunk_keys_array_by_shpere(clip_spheres.old_sphere, neighbour_offest.clone())
                .drain(..)
        {
            chunks_to_remove.insert(key);
        }

        for key in
            find_chunk_keys_array_by_shpere(clip_spheres.new_sphere, neighbour_offest.clone())
                .drain(..)
        {
            chunks_to_remove.remove(&key);
        }
    }

    for chunk_key in chunks_to_remove.into_iter() {
        if let Some(entity) = collider_manager.entities.remove(&chunk_key) {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ColliderSystem {
    ColliderTask,
    ColliderSpawn,
    ColliderDespawn,
}

pub struct TerrainPhysicsPlugin;

impl Plugin for TerrainPhysicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ColliderManager {
            entities: HashMap::default(),
        })
        .insert_resource(ColliderTasksManager::default())
        .insert_resource(ColliderUpdateTasksManager::default())
        .add_systems(
            PreUpdate,
            server_update_collider_task_system.in_set(ColliderSystem::ColliderTask),
        )
        .add_systems(
            PreUpdate,
            spawn_collider
                .in_set(ColliderSystem::ColliderSpawn)
                .after(ColliderSystem::ColliderTask),
        )
        .add_systems(PreUpdate, update_codiller)
        .add_systems(
            Last,
            despawn_collider.in_set(ColliderSystem::ColliderDespawn),
        );
    }
}

/**
 * 通过包含邻居的体素数据 获取碰撞体
 */
pub fn gen_collider(voxels: Vec<Voxel>) -> Option<Collider> {
    type SampleShape =
        ConstShape3u32<CHUNK_SIZE_ADD_2_U32, CHUNK_SIZE_ADD_2_U32, CHUNK_SIZE_ADD_2_U32>;
    let mut buffer = GreedyQuadsBuffer::new(SampleShape::SIZE as usize);
    let faces: [block_mesh::OrientedBlockFace; 6] = RIGHT_HANDED_Y_UP_CONFIG.faces;
    greedy_quads(
        &voxels,
        &SampleShape {},
        [0; 3],
        [
            (CHUNK_SIZE + 1) as u32,
            (CHUNK_SIZE + 1) as u32,
            (CHUNK_SIZE + 1) as u32,
        ],
        &faces,
        &mut buffer,
    );
    let num_indices = buffer.quads.num_quads() * 6;
    let num_vertices = buffer.quads.num_quads() * 4;
    if num_indices == 0 {
        return None;
    }
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);

    for (_, (group, face)) in buffer
        .quads
        .groups
        .as_ref()
        .into_iter()
        .zip(faces.into_iter())
        .enumerate()
    {
        for quad in group.into_iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0));
            normals.extend_from_slice(&face.quad_mesh_normals());
        }
    }
    let collider_vertices: Vec<Vec3> = positions.iter().cloned().map(|p| Vec3::from(p)).collect();
    let collider_indices: Vec<[u32; 3]> = indices.chunks(3).map(|i| [i[0], i[1], i[2]]).collect();
    let collider = Collider::trimesh(collider_vertices, collider_indices);
    Some(collider)
}

#[derive(Debug, Resource, Default)]
pub struct ColliderUpdateTasksManager {
    pub tasks: Vec<Task<(Entity, ChunkKey, Vec<Voxel>)>>,
}

fn update_codiller(
    mut commands: Commands,
    mut collider_update_tasks_manager: ResMut<ColliderUpdateTasksManager>,
    mut query: Query<(Entity, &mut Collider, &TerrainPhysics)>,
    mut collider_manager: ResMut<ColliderManager>,
) {
    let len = collider_update_tasks_manager.tasks.len().min(256);
    for ele in collider_update_tasks_manager.tasks.drain(..len) {
        match futures_lite::future::block_on(futures_lite::future::poll_once(ele)) {
            Some((entity, chunk_key, voxels)) => {
                if let Ok((_, mut collider, _)) = query.get_mut(entity) {
                    if let Some(c) = gen_collider(voxels) {
                        *collider = c;
                    } else {
                        if let Some(entity) = collider_manager.entities.remove(&chunk_key) {
                            commands.entity(entity).despawn();
                        }
                    }
                }
            }
            None => {}
        }
    }
}
