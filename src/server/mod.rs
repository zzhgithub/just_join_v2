use bevy::prelude::{Commands, Entity, EventReader, Query, Res, ResMut, Transform, Vec3, With};
use bevy_rapier3d::{
    prelude::{RapierContext, RapierRigidBodyHandle},
    rapier::prelude::RigidBodyMassProps,
};
use bevy_renet::renet::{transport::NetcodeServerTransport, RenetServer, ServerEvent};
use renet_visualizer::RenetServerVisualizer;

use crate::{
    client::{client_channel::ClientChannel, player_input::PlayerInput},
    server::{
        player::server_create_player, message_def::{server_messages::ServerMessages, ServerChannel},
    },
    users::Username,
    voxel_world::{
        map_database::MapDataBase,
        player_state::{PlayerOntimeState, PlayerState, StorgePlayerState},
    },
};

use self::{
    player::{PitchValue, Player, ServerLobby, YawValue}, message_def::networked_entities::NetworkedEntities,
};

pub mod async_chunk;
pub mod chunk;
pub mod object_filing;
pub mod player;
pub mod terrain_physics;
pub mod message_def;

/**
 * 处理client连接获取断开时的操作
 */
#[allow(clippy::too_many_arguments)]
pub fn server_connect_system(
    mut commands: Commands,
    mut server_events: EventReader<ServerEvent>,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
    players: Query<(Entity, &Player, &Transform, &PlayerOntimeState)>,
    mut server: ResMut<RenetServer>,
    mut server_lobby: ResMut<ServerLobby>,
    transport: Res<NetcodeServerTransport>,
    mut map_database: ResMut<MapDataBase>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                let user_data = transport.user_data(*client_id).unwrap();
                let username = Username::from_user_data(&user_data).0;
                println!("Player {}|{} connected.", client_id, username);
                if server_lobby.names.contains(&username) {
                    // 相同用户名重复登录
                    server.remove_connection(*client_id);
                    println!("Player {}|{} 重复登录 已经被阻止.", client_id, username);
                    continue;
                }
                server_lobby.names.insert(username.clone());
                visualizer.add_client(*client_id);
                // 1. 先通知 当前连接 其他的已经存在的用户数据
                for (entity, player, transform, _) in players.iter() {
                    let translation: [f32; 3] = transform.translation.into();
                    let message = bincode::serialize(&ServerMessages::PlayerCreate {
                        id: player.id,
                        entity,
                        translation,
                        username: player.username.clone(),
                    })
                    .unwrap();
                    server.send_message(*client_id, ServerChannel::ServerMessages, message);
                }
                // 2. 创建这个用户并(注意这里不用mesh 直接创建 一个物理对象就可以了。因为服务器不关心物体的姿态)
                let transform = Transform::from_xyz(0., 60., 0.);
                // -- 获取 到用户的信息
                let mut player_state;
                if let Some(state) = map_database.get_player_state(username.clone()) {
                    // 获取历史数据
                    player_state = state;
                } else {
                    // 第一次新建数据
                    player_state = PlayerState::default();
                    player_state.position = [0., 60., 0.];
                }

                let player_entity =
                    server_create_player(&mut commands, player_state, *client_id, username.clone());
                // 角色进入游戏大厅缓存中
                server_lobby.players.insert(*client_id, player_entity);
                // 3. 通知全部客户端知道
                let translation: [f32; 3] = transform.translation.into();
                let message = bincode::serialize(&ServerMessages::PlayerCreate {
                    id: *client_id,
                    entity: player_entity,
                    translation,
                    username: username.clone(),
                })
                .unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                if let bevy_renet::renet::DisconnectReason::DisconnectedByServer = reason {
                    continue;
                }
                visualizer.remove_client(*client_id);
                println!("Player {} disconnected: {}", client_id, reason);
                // 告诉所有人减少了一个用户
                if let Some(player_entity) = server_lobby.players.remove(client_id) {
                    // 在用户断开连接是保存用户数据到数据库
                    if let Ok((_, player, tf, state)) = players.get(player_entity) {
                        let mut save_state = state.0.clone();
                        save_state.position =
                            [tf.translation.x, tf.translation.y, tf.translation.z];
                        server_lobby.names.remove(&player.username.clone());
                        map_database.save_player_state(player.username.clone(), save_state);
                    }
                    commands.entity(player_entity).despawn();
                }

                let message =
                    bincode::serialize(&ServerMessages::PlayerRemove { id: *client_id }).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
        }
    }
}

pub fn deal_message_system(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    lobby: ResMut<ServerLobby>,
    mut context: ResMut<RapierContext>,
    query: Query<(Entity, &RapierRigidBodyHandle), With<Player>>,
) {
    let xz = Vec3::new(1.0, 0.0, 1.0);
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input) {
            let player_input: PlayerInput = bincode::deserialize(&message).unwrap();
            match player_input {
                PlayerInput::MOVE(vec3) => {
                    if let Some(player_entity) = lobby.players.get(&client_id) {
                        if let Ok((_, handle)) = query.get(*player_entity) {
                            if let Some(body) = context.bodies.get_mut(handle.0) {
                                let mass_props: &RigidBodyMassProps = body.mass_properties();
                                let effective_mass = mass_props.effective_mass();
                                let velocity: Vec3 = (*body.linvel()).into();
                                // 作用冲量
                                body.apply_impulse(
                                    ((vec3 - velocity * xz) * effective_mass.x).into(),
                                    true,
                                );
                            }
                        }
                    }
                }
                PlayerInput::YAW(yaw) => {
                    if let Some(player_entity) = lobby.players.get(&client_id) {
                        commands.entity(*player_entity).insert(YawValue(yaw));
                    }
                }
                PlayerInput::PITCH(patch) => {
                    if let Some(player_entity) = lobby.players.get(&client_id) {
                        commands.entity(*player_entity).insert(PitchValue(patch));
                    }
                }
            }
        }
    }
}

// 同步玩家角色的位置 头部

pub fn sync_body_and_head(
    players: Query<(Entity, &Player, &Transform, &YawValue, &PitchValue)>,
    mut server: ResMut<RenetServer>,
) {
    let mut networked_entities = NetworkedEntities::default();
    for (_, player, transform, yaw_value, pitch_value) in players.iter() {
        networked_entities.client_ids.push(player.id);
        networked_entities
            .translations
            .push(transform.translation.into());
        networked_entities.yaws.push(yaw_value.0);
        networked_entities.pitch.push(pitch_value.0);
    }
    let sync_message = bincode::serialize(&networked_entities).unwrap();
    server.broadcast_message(ServerChannel::NetworkedEntities, sync_message);
}
