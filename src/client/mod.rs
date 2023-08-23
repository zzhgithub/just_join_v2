use bevy::prelude::{
    Assets, Commands, DespawnRecursiveExt, Mesh, Quat, Query, Res, ResMut, StandardMaterial,
    Transform, Without,
};
use bevy_renet::renet::{transport::NetcodeClientTransport, RenetClient};

use crate::{
    client::player::PlayerInfo,
    server::{
        networked_entities::NetworkedEntities, server_channel::ServerChannel,
        server_messages::ServerMessages,
    },
};

use self::player::{
    client_create_player,
    controller::{HeadTag, YawTag},
    ClientLobby,
};

pub mod chunk_query;
pub mod client_channel;
pub mod console_commands;
pub mod mesh_display;
pub mod player;
pub mod player_input;
pub mod ray_cast;
pub mod state_manager;
pub mod voxels;

// 同步创建或者删除角色
pub fn client_sync_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut client: ResMut<RenetClient>,
    transport: Res<NetcodeClientTransport>,
    mut lobby: ResMut<ClientLobby>,
) {
    let client_id = transport.client_id();
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let server_message: ServerMessages = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessages::PlayerCreate {
                entity,
                id,
                translation,
            } => {
                println!("Player {} connected.", id);
                // 创建物体的人物实体 只有mesh

                let (client_entity, yaw, head) = client_create_player(
                    &mut commands,
                    Transform::from_xyz(translation[0], translation[1], translation[2]),
                    client_id,
                    materials.as_mut(),
                    meshes.as_mut(),
                    client_id == id,
                );

                let player_info = PlayerInfo {
                    server_entity: entity,
                    client_entity: client_entity,
                };
                lobby.players.insert(id, player_info);
                // 这记录脖子和头部的对应关系
                lobby.yaws.insert(id, yaw);
                lobby.pitch.insert(id, head);
                // 如果就是当前数据的话 绑定 clip_shpere 和 相机
                if client_id == id {
                    // 这里创造一下球体？
                }
            }
            ServerMessages::PlayerRemove { id } => {
                println!("Player {} disconnected.", id);
                lobby.yaws.remove(&id);
                lobby.pitch.remove(&id);
                if let Some(PlayerInfo {
                    server_entity: _,
                    client_entity,
                }) = lobby.players.remove(&id)
                {
                    commands.entity(client_entity).despawn_recursive();
                }
            }
        }
    }
}

// 同步角色移动或者头部移动
pub fn client_sync_players_state(
    mut commands: Commands,
    mut yaw_query: Query<(&YawTag, &mut Transform)>,
    mut patch_query: Query<(&HeadTag, &mut Transform), Without<YawTag>>,
    mut client: ResMut<RenetClient>,
    lobby: Res<ClientLobby>,
) {
    while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities) {
        let server_message: NetworkedEntities = bincode::deserialize(&message).unwrap();
        let NetworkedEntities {
            client_ids,
            translations,
            yaws,
            pitch,
        } = server_message;
        // 对网络中的物体进行位移
        for i in 0..client_ids.len() {
            let client_id = client_ids[i];
            if let Some(PlayerInfo {
                client_entity,
                server_entity: _,
            }) = lobby.players.get(&client_id)
            {
                let translation = translations[i].into();
                let transform = Transform {
                    translation,
                    ..Default::default()
                };
                commands.entity(*client_entity).insert(transform);
            }

            if let Some(entity) = lobby.yaws.get(&client_id) {
                if let Ok((_, mut tfr)) = yaw_query.get_mut(*entity) {
                    tfr.rotation = Quat::from_rotation_y(yaws[i]);
                }
            }
            if let Some(entity) = lobby.pitch.get(&client_id) {
                if let Ok((_, mut tfr)) = patch_query.get_mut(*entity) {
                    tfr.rotation = Quat::from_rotation_x(pitch[i]);
                }
            }
        }
    }
}
