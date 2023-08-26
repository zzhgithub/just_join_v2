use bevy::{
    prelude::{Commands, Component, Entity, Resource, Transform, Vec3},
    transform::TransformBundle,
    utils::HashMap,
};
use bevy_rapier3d::prelude::{
    Ccd, Collider, ColliderMassProperties, LockedAxes, RigidBody, Sleeping,
};

use crate::voxel_world::player_state::{PlayerOntimeState, PlayerState};

#[derive(Debug, Component)]
pub struct Player {
    pub id: u64,
    pub username: String,
}

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    // client_id ==> entity
    pub players: HashMap<u64, Entity>,
}

pub fn server_create_player(
    commands: &mut Commands,
    player_state: PlayerState,
    client_id: u64,
    username: String,
) -> Entity {
    let pos = player_state.position.clone();
    let transform = Transform::from_xyz(pos[0], pos[1], pos[2]);
    commands
        .spawn(Player {
            id: client_id,
            username: username,
        })
        .insert(TransformBundle::from(transform))
        .insert(RigidBody::Dynamic)
        .insert(Sleeping::default())
        .insert(ColliderMassProperties::Mass(300.0))
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Collider::capsule(
            (-0.5 * 1.7 * Vec3::Y).into(),
            (0.5 * (1.7 - 0.9) * Vec3::Y).into(),
            0.3,
        ))
        .insert(Ccd::enabled())
        .insert(YawValue::default())
        .insert(PitchValue::default())
        .insert(PlayerOntimeState(player_state))
        .id()
}

#[derive(Debug, Component, Default)]
pub struct YawValue(pub f32);

#[derive(Debug, Component, Default)]
pub struct PitchValue(pub f32);
