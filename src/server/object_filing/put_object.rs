use bevy::prelude::Query;
use bevy_renet::renet::RenetServer;

use crate::{
    server::{
        message_def::{tool_bar_message::ToolBarMessage, ServerChannel},
        player::ServerLobby,
    },
    voxel_world::player_state::PlayerOnTimeState,
};

/// 放置方块。如果成功修改toolbar并发送消息。如果失败的情况下 直接返回false
pub fn put_object(
    client_id: u64,
    server_lobby: &ServerLobby,
    query: &mut Query<&mut PlayerOnTimeState>,
    active_index: usize,
    staff_id: usize,
    server: &mut RenetServer,
) -> bool {
    if let Some(entity) = server_lobby.players.get(&client_id) {
        if let Ok(mut player_state) = query.get_mut(*entity) {
            if let Some((index, data, num)) = player_state.0.use_staff(active_index, staff_id, 1) {
                // 找到位置并摆放
                // 发送消息销毁对象
                let message = bincode::serialize(&ToolBarMessage::SyncToolbar {
                    index: index,
                    staff_id: data,
                    num: num,
                })
                .unwrap();
                server.send_message(client_id, ServerChannel::ToolBarMessage, message);
                return true;
            }
        }
    }
    return false;
}
