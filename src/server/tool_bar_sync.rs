use bevy_renet::renet::RenetServer;

use crate::voxel_world::player_state::PlayerState;

use super::message_def::{tool_bar_message::ToolBarMessage, ServerChannel};

// 同步全部toolbar信息
pub fn send_all_tool_bar(client_id: u64, server: &mut RenetServer, player_state: PlayerState) {
    for i in 0..player_state.toolbar.len() {
        if let (Some(staff_id), num) = player_state.toolbar[i] {
            let message = bincode::serialize(&ToolBarMessage::SyncToolbar {
                index: i,
                staff_id: Some(staff_id),
                num: num,
            })
            .unwrap();
            server.send_message(client_id, ServerChannel::ToolBarMessage, message);
        }
    }
}
