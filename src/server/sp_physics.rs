use bevy::{
    prelude::{
        Assets, Commands, Entity, Event, EventReader, EventWriter, GlobalTransform, Mesh, Plugin,
        PreUpdate, Res, ResMut, Resource, Vec3,
    },
    tasks::{AsyncComputeTaskPool, Task},
    utils::{HashMap, HashSet},
};
use bevy_rapier3d::prelude::{Collider, CollisionGroups, Group, RigidBody};

use crate::{
    client::sp_mesh_display::get_sp_tfr,
    common::ServerClipSpheres,
    voxel_world::{
        chunk::{find_chunk_keys_array_by_sphere, generate_offset_array, ChunkKey},
        chunk_map::ChunkMap,
        voxel::VoxelDirection,
        voxel_mesh::{MeshMateData, VoxelMeshStorge, VOXEL_MESH_MAP},
    },
    PY_DISTANCE,
};

use super::terrain_physics::TerrainPhysics;

// 管理特殊的物理对象
#[derive(Debug, Clone, Resource)]
pub struct SpPhysicsManager {
    pub entities: HashMap<ChunkKey, HashMap<usize, Entity>>,
}

impl SpPhysicsManager {
    pub fn has_data(&self, chunk_key: ChunkKey, index: usize) -> bool {
        if let Some(inner_map) = self.entities.get(&chunk_key) {
            return inner_map.contains_key(&index);
        }
        false
    }

    pub fn insert(&mut self, chunk_key: ChunkKey, index: usize, entity: Entity) {
        if !self.entities.contains_key(&chunk_key) {
            self.entities.insert(chunk_key, HashMap::new());
        }
        if let Some(inner_map) = self.entities.get_mut(&chunk_key) {
            inner_map.insert(index, entity);
        }
    }

    pub fn remove(&mut self, chunk_key: ChunkKey, index: usize) -> Option<Entity> {
        if self.has_data(chunk_key, index) {
            if let Some(inner_map) = self.entities.get_mut(&chunk_key) {
                return inner_map.remove(&index);
            }
        }
        None
    }
}

impl Default for SpPhysicsManager {
    fn default() -> Self {
        Self {
            entities: Default::default(),
        }
    }
}

// 处理生成物理引擎任务
#[derive(Debug, Resource)]
pub struct SpPhysicsTasks {
    pub tasks: Vec<Task<(ChunkKey, u32, MeshMateData, VoxelDirection)>>,
}

impl Default for SpPhysicsTasks {
    fn default() -> Self {
        Self {
            tasks: Default::default(),
        }
    }
}

pub struct SpPhysicsPlugin;

impl Plugin for SpPhysicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<DespawnSpEvent>();
        app.insert_resource(SpPhysicsManager::default());
        app.insert_resource(SpPhysicsTasks::default());
        // 处理生成和销毁
        app.add_systems(
            PreUpdate,
            (
                gen_sp_physics_tasks,
                deal_gen_collider,
                despawn_sp_event_gen,
                deal_events,
            ),
        );
    }
}

// 生成物理模型的tasks
fn gen_sp_physics_tasks(
    sp_physics_manager: Res<SpPhysicsManager>,
    mut sp_physics_tasks: ResMut<SpPhysicsTasks>,
    chunk_map: Res<ChunkMap>,
    server_clip_spheres: Res<ServerClipSpheres>,
    voxel_mesh_storge: Res<VoxelMeshStorge>,
) {
    let pool = AsyncComputeTaskPool::get();
    let mut keys: HashSet<ChunkKey> = HashSet::new();
    for (_client_id, clip_spheres) in server_clip_spheres.clip_spheres.iter() {
        for chunk_key in find_chunk_keys_array_by_sphere(
            clip_spheres.new_sphere,
            generate_offset_array(PY_DISTANCE),
        )
        .drain(..)
        {
            if !keys.contains(&chunk_key) {
                keys.insert(chunk_key);
                // 在一次的system中没有的情况才处理
                if let Some(voxels) = chunk_map.get(chunk_key) {
                    // 存在地图数据
                    for i in 0..voxels.len() {
                        if VOXEL_MESH_MAP.contains_key(&voxels[i].id)
                            && !sp_physics_manager.has_data(chunk_key, i)
                        // 原来没有数据
                        {
                            // 存在SP的类型的方块
                            if let Some(meta_data) = voxel_mesh_storge.data.get(&voxels[i].id) {
                                let ret = (
                                    chunk_key.clone(),
                                    i as u32,
                                    meta_data.clone(),
                                    voxels[i].direction,
                                );
                                let task = pool.spawn(async move { ret });
                                sp_physics_tasks.tasks.push(task);
                            }
                        }
                    }
                }
            }
        }
    }
}

// 生成碰撞体数据
fn deal_gen_collider(
    mut commands: Commands,
    mut sp_physics_manager: ResMut<SpPhysicsManager>,
    mut sp_physics_tasks: ResMut<SpPhysicsTasks>,
    mesh_assets: Res<Assets<Mesh>>,
) {
    let len = sp_physics_tasks.tasks.len().min(256);
    for ele in sp_physics_tasks.tasks.drain(..len) {
        if let Some((chunk_key, index, mate_data, direction)) =
            futures_lite::future::block_on(futures_lite::future::poll_once(ele))
        {
            if !sp_physics_manager.has_data(chunk_key, index as usize) {
                // FIXME: 后续这里要其他的地方接管 碰撞体的生成
                if let Some(mesh) = mesh_assets.get(&mate_data.vox_list[0].clone()) {
                    if let Some(collider) = get_collider_by_mesh(mesh) {
                        println!("这里生成碰撞体");
                        let entity = commands
                            .spawn((
                                TerrainPhysics,
                                get_sp_tfr(chunk_key.clone(), index.clone(), direction),
                                GlobalTransform::default(),
                            ))
                            .insert(RigidBody::Fixed)
                            .insert(collider)
                            .insert(CollisionGroups::new(
                                Group::GROUP_1,
                                Group::GROUP_2 | Group::GROUP_3,
                            ))
                            .id();
                        sp_physics_manager.insert(chunk_key, index as usize, entity);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Event)]
pub struct DespawnSpEvent {
    pub chunk_key: ChunkKey,
    pub index: usize,
}
// 删除多余的数据
fn despawn_sp_event_gen(
    sp_physics_manager: Res<SpPhysicsManager>,
    server_clip_spheres: Res<ServerClipSpheres>,
    mut event_writer: EventWriter<DespawnSpEvent>,
) {
    // FIXME: 这里是重复的代码 需要重新整理
    let neighbour_offest = generate_offset_array(PY_DISTANCE);
    let mut chunks_to_remove = HashSet::new();
    for (_client_id, clip_spheres) in server_clip_spheres.clip_spheres.iter() {
        for key in
            find_chunk_keys_array_by_sphere(clip_spheres.old_sphere, neighbour_offest.clone())
                .drain(..)
        {
            chunks_to_remove.insert(key);
        }

        for key in
            find_chunk_keys_array_by_sphere(clip_spheres.new_sphere, neighbour_offest.clone())
                .drain(..)
        {
            chunks_to_remove.remove(&key);
        }
    }

    for chunk_key in chunks_to_remove.into_iter() {
        if let Some(inner_map) = sp_physics_manager.entities.get(&chunk_key) {
            for (index, _) in inner_map {
                event_writer.send(DespawnSpEvent {
                    chunk_key,
                    index: index.clone(),
                });
            }
        }
    }
}

fn deal_events(
    mut commands: Commands,
    mut sp_physics_manager: ResMut<SpPhysicsManager>,
    mut event_reader: EventReader<DespawnSpEvent>,
) {
    for event in event_reader.iter() {
        if let Some(entity) = sp_physics_manager.remove(event.chunk_key, event.index) {
            // println!("这里销毁了碰撞体");
            commands.entity(entity).despawn();
        }
    }
}

// 通过mesh生成碰撞体
fn get_collider_by_mesh(mesh: &Mesh) -> Option<Collider> {
    let mut collider_vertices: Vec<Vec3> = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap()
        .to_vec()
        .iter()
        .cloned()
        .map(Vec3::from)
        .collect();
    let indices: Vec<u32> = mesh.indices().unwrap().iter().map(|x| x as u32).collect();
    let collider_indices: Vec<[u32; 3]> = indices.chunks(3).map(|i| [i[0], i[1], i[2]]).collect();
    // println!("{:?}", collider_vertices);
    // println!("{:?}", collider_indices);
    #[cfg(feature = "headless")]
    {
        collider_vertices = collider_vertices
            .iter()
            .map(|x| *x * Vec3::splat(1.0 / 32.0))
            .collect();
    }
    let collider = Collider::trimesh(collider_vertices, collider_indices);
    Some(collider)
}
