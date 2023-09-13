// 接受处理 物体被丢弃的消息
use bevy::{
    prelude::{Commands, Component, Entity, Plugin, Query, Res, ResMut, Transform, Update},
    time::{Time, Timer, TimerMode},
};
use bevy_rapier3d::prelude::ExternalImpulse;
use bevy_renet::renet::RenetServer;

use crate::{
    client::message_def::{user_command::UserCommandMessage, ClientChannel},
    server::{player::ServerLobby, tool_bar_sync::send_all_tool_bar},
    staff::StaffInfoStroge,
    tools::vec3_to_chunk_key_any_xyz,
    voxel_world::player_state::PlayerOnTimeState,
};

use super::gen_filled_object;

#[derive(Debug, Component, Clone)]
pub struct ThrowObject(pub Timer);

pub fn deal_with_throw_object(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    server_lobby: Res<ServerLobby>,
    mut query: Query<(Entity, &Transform, &mut PlayerOnTimeState)>,
    staff_info_stroge: Res<StaffInfoStroge>,
) {
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Command) {
            if let Some(entity) = server_lobby.players.get(&client_id) {
                if let Ok((_, trf, mut player_state)) = query.get_mut(*entity) {
                    let message: UserCommandMessage = bincode::deserialize(&message).unwrap();
                    match message {
                        UserCommandMessage::Throw {
                            index,
                            staff_id,
                            forward,
                        } => {
                            if let Some(staff) = staff_info_stroge.get(staff_id) {
                                // 判断是否可以 丢弃物品？
                                if player_state.0.use_staff(index, staff_id, 1) != None {
                                    // 同步 toolbar
                                    send_all_tool_bar(
                                        client_id,
                                        &mut server,
                                        player_state.0.clone(),
                                    );
                                    // 生成 丢弃物
                                    let (chunk_key, _) = vec3_to_chunk_key_any_xyz(trf.translation);
                                    let throw_object = gen_filled_object(
                                        &mut commands,
                                        chunk_key,
                                        trf.translation,
                                        staff,
                                    );
                                    // 添加throw组件。 和额外冲量
                                    commands
                                        .entity(throw_object)
                                        .insert(ThrowObject(Timer::new(
                                            bevy::utils::Duration::from_millis(1000 * 1),
                                            TimerMode::Once,
                                        )))
                                        .insert(ExternalImpulse {
                                            impulse: forward * 8.0 * 300.0,
                                            ..Default::default()
                                        });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// 对 throw物体进行 延时计算
pub fn deal_throw_delayed(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ThrowObject)>,
) {
    for (entity, mut throw_object) in query.iter_mut() {
        throw_object.0.tick(time.delta());
        if throw_object.0.finished() {
            commands.entity(entity).remove::<ThrowObject>();
        }
    }
}

pub struct ThrowObjectPlugin;

impl Plugin for ThrowObjectPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, (deal_with_throw_object, deal_throw_delayed));
    }
}
