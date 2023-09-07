use std::f32::consts::TAU;

use ahash::HashSet;
use bevy::{
    prelude::{
        in_state, Color, Commands, Component, Entity, Gizmos, IntoSystemConfigs,
        MaterialMeshBundle, Plugin, Query, Res, ResMut, Resource, Transform, Update, Vec3,
    },
    time::Time,
    utils::HashMap,
};
use bevy_renet::renet::RenetClient;
use bevy_sprite3d::{Sprite3d, Sprite3dParams};

use crate::{
    server::message_def::{filled_object_message::FilledObjectMessage, ServerChannel},
    staff::StaffInfoStroge,
    CLIENT_DEBUG,
};

use super::{
    state_manager::GameState,
    voxels::{
        mesh::gen_one_volex_mesh, mesh_material::MaterialStorge,
        voxel_materail_config::MaterailConfiguration,
    },
};

#[derive(Debug, Clone, Resource, Default)]
pub struct FilledObjectPool {
    // 服务端 entity 和 客户端 entity 对应表
    pub entities_map: HashMap<Entity, Entity>,
}

#[derive(Debug, Clone, Component)]
pub struct FilledObjectCommpent;

pub struct ClientFilledObjectnPlugin;

impl Plugin for ClientFilledObjectnPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(FilledObjectPool::default());
        app.add_systems(
            Update,
            (rotate_filled_objects, sync_filled_objects)
                .run_if(in_state(GameState::Game))
                .run_if(bevy_renet::transport::client_connected()),
        );
    }
}

//FIXME: 存在问题 旋转物体的mesh
fn rotate_filled_objects(
    mut query: Query<(&FilledObjectCommpent, &mut Transform)>,
    timer: Res<Time>,
    mut gizmos: Gizmos,
) {
    for (_, mut transform) in &mut query {
        transform.rotate_y(0.3 * TAU * timer.delta_seconds());
        if CLIENT_DEBUG {
            gizmos.circle(transform.translation, Vec3::Y, 0.1, Color::YELLOW);
            gizmos.ray(transform.translation, Vec3::Y, Color::YELLOW);
        }
    }
}

fn sync_filled_objects(
    mut commands: Commands,
    mut filled_object_pool: ResMut<FilledObjectPool>,
    mut client: ResMut<RenetClient>,
    staff_info_stroge: Res<StaffInfoStroge>,
    materials: Res<MaterialStorge>,
    material_config: Res<MaterailConfiguration>,
    // mut mesh_assets: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &FilledObjectCommpent, &mut Transform)>,
    mut sprite_params: Sprite3dParams,
) {
    while let Some(message) = client.receive_message(ServerChannel::FilledObjectMessage) {
        let message: FilledObjectMessage = bincode::deserialize(&message).unwrap();
        match message {
            FilledObjectMessage::SyncFilledObject(objs) => {
                let mut new_set: HashSet<Entity> = HashSet::default();
                if objs.is_empty() {
                    // 全部清空
                } else {
                    for (server_entity, staff_id, pos) in objs.iter() {
                        new_set.insert(server_entity.clone());
                        if let Some(client_entity) =
                            filled_object_pool.entities_map.get(server_entity)
                        {
                            // 已经存在 修改位置
                            if let Ok((_, _, mut trf)) = query.get_mut(client_entity.clone()) {
                                trf.translation = Vec3::new(pos[0], pos[1], pos[2]);
                            }
                        } else {
                            // 不存在 创建 实体
                            if let Some(staff) = staff_info_stroge.get(staff_id.clone()) {
                                match staff.staff_type {
                                    crate::staff::StaffType::Voxel(voxel) => {
                                        // 生成一个mesh 并且渲染
                                        if let Some(render_mesh) =
                                            gen_one_volex_mesh(voxel, material_config.clone())
                                        {
                                            let mesh_handle = sprite_params.meshes.add(render_mesh);
                                            let client_entity = commands
                                                .spawn(MaterialMeshBundle {
                                                    transform: Transform {
                                                        translation: Vec3::new(
                                                            pos[0], pos[1], pos[2],
                                                        ),
                                                        scale: Vec3::splat(0.1),
                                                        ..Default::default()
                                                    },
                                                    mesh: mesh_handle.clone(),
                                                    material: materials.0.clone(),
                                                    ..Default::default()
                                                })
                                                .insert(FilledObjectCommpent)
                                                .id();
                                            filled_object_pool
                                                .entities_map
                                                .insert(server_entity.clone(), client_entity);
                                        }
                                    }
                                    _ => {
                                        // 生成贴图数据
                                        let client_entity = commands
                                            .spawn(
                                                Sprite3d {
                                                    image: staff.icon,
                                                    pixels_per_metre: 400.,
                                                    partial_alpha: true,
                                                    unlit: true,
                                                    transform: Transform::from_xyz(
                                                        pos[0], pos[1], pos[2],
                                                    ),
                                                    double_sided: true,
                                                    // pivot: Some(Vec2::new(0.5, 0.5)),
                                                    ..Default::default()
                                                }
                                                .bundle(&mut sprite_params),
                                            )
                                            .insert(FilledObjectCommpent)
                                            .id();
                                        filled_object_pool
                                            .entities_map
                                            .insert(server_entity.clone(), client_entity);
                                    }
                                }
                            }
                        }
                    }
                }
                // 只有更改了才处理
                let mut delete_keys: HashSet<Entity> = HashSet::default();
                // 清除多余数据
                for (key, _) in filled_object_pool.entities_map.iter() {
                    if !new_set.contains(key) {
                        delete_keys.insert(key.clone());
                    }
                }
                for key in delete_keys.iter() {
                    if let Some(client_entity) = filled_object_pool.entities_map.remove(key) {
                        commands.entity(client_entity).despawn();
                    }
                }
            }
        }
    }
}

pub fn setdown_filled_object(
    mut commands: Commands,
    mut filled_object_pool: ResMut<FilledObjectPool>,
) {
    for (_, entity) in filled_object_pool.entities_map.clone() {
        commands.entity(entity).despawn();
    }
    filled_object_pool.entities_map = HashMap::new();
}
