// 穿透检查

use bevy::prelude::{
    Commands, Component, Entity, Plugin, PreUpdate, Query, Res, ResMut, Transform, Update, Vec3,
    Without,
};
use bevy_rapier3d::{
    prelude::{RapierContext, RapierRigidBodyHandle},
    rapier::prelude::RigidBodyType,
};

use crate::{
    tools::{chunk_key_any_xyz_to_vec3, pos_to_center, vec3_to_chunk_key_any_xyz},
    voxel_world::{chunk_map::ChunkMap, voxel::Voxel},
};

#[derive(Debug, Clone, Copy, Component)]
pub struct CossTroughCheck;

#[derive(Debug, Clone, Copy, Component)]
pub struct CossTroughFixed(Vec3);

pub struct CossTroughCheckPlugin;

impl Plugin for CossTroughCheckPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PreUpdate, cross_through_check);
        app.add_systems(Update, cross_through_fixed);
    }
}

fn cross_through_check(
    mut commands: Commands,
    mut context: ResMut<RapierContext>,
    query: Query<
        (Entity, &RapierRigidBodyHandle, &Transform, &CossTroughCheck),
        Without<CossTroughFixed>,
    >,
    chunk_map: Res<ChunkMap>,
) {
    for (entity, body_handle, trf, _) in query.iter() {
        let (chunk_key, xyz) = vec3_to_chunk_key_any_xyz(pos_to_center(trf.translation));
        if let Some(test_voxel) = chunk_map.get_block(chunk_key, xyz) {
            if test_voxel.id != Voxel::EMPTY.id {
                if let Some((new_chunk_key, new_xyz)) =
                    chunk_map.find_closest_block_y(chunk_key, xyz, Voxel::EMPTY.id)
                {
                    if let Some(body) = context.bodies.get_mut(body_handle.0) {
                        body.set_body_type(RigidBodyType::KinematicPositionBased, true);
                        let mut pos = chunk_key_any_xyz_to_vec3(new_chunk_key, new_xyz);
                        pos.x = trf.translation.x;
                        pos.z = trf.translation.z;
                        commands.entity(entity).insert(CossTroughFixed(pos));
                    }
                }
            }
        }
    }
}

fn cross_through_fixed(
    mut commands: Commands,
    mut context: ResMut<RapierContext>,
    mut query: Query<(
        Entity,
        &RapierRigidBodyHandle,
        &mut Transform,
        &CossTroughFixed,
    )>,
) {
    for (entity, body_handle, mut trf, fixed) in query.iter_mut() {
        if let Some(body) = context.bodies.get_mut(body_handle.0) {
            // println!("修复了物体 to {}", fixed.0);
            trf.translation = fixed.0;
            body.set_body_type(RigidBodyType::Dynamic, true);
            commands.entity(entity).remove::<CossTroughFixed>();
        }
    }
}
