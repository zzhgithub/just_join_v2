use bevy::{
    prelude::{Commands, Component, Entity, Resource, Transform, Vec3},
    transform::TransformBundle,
    utils::HashMap,
};
use bevy_rapier3d::prelude::{Collider, LockedAxes, RigidBody, Velocity};

#[derive(Debug, Component)]
pub struct Player {
    pub id: u64,
}

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    // client_id ==> entity
    pub players: HashMap<u64, Entity>,
}

pub fn server_create_player(
    commands: &mut Commands,
    transform: Transform,
    client_id: u64,
) -> Entity {
    commands
        .spawn(Player { id: client_id })
        .insert(TransformBundle::from(transform))
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y)
        .insert(Velocity::default())
        // TODO: 这里暂时写死后面要变成配置化的数据
        .insert(Collider::capsule(
            (-0.5 * 1.9 * Vec3::Y).into(),
            (0.5 * (1.9 - 0.9) * Vec3::Y).into(),
            0.3,
        ))
        .insert(YawValue::default())
        .insert(PitchValue::default())
        .id()
}

#[derive(Debug, Component, Default)]
pub struct YawValue(pub f32);

#[derive(Debug, Component, Default)]
pub struct PitchValue(pub f32);
