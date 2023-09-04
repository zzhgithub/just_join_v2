use bevy::prelude::{in_state, IntoSystemConfigs, Plugin, Res, ResMut, Update};
use bevy_renet::renet::RenetClient;

use crate::{
    server::message_def::{tool_bar_message::ToolBarMessage, ServerChannel},
    staff::StaffInfoStroge,
};

use super::{state_manager::GameState, ui::tool_bar::ToolBar};

// 同步 toolbar信息 相关插件
pub struct ToolBarSyncPlugin;

impl Plugin for ToolBarSyncPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            sync_toolbar_message
                .run_if(in_state(GameState::Game))
                .run_if(bevy_renet::transport::client_connected()),
        );
    }
}

fn sync_toolbar_message(
    mut client: ResMut<RenetClient>,
    mut tool_bar_data: ResMut<ToolBar>,
    staff_infos: Res<StaffInfoStroge>,
) {
    let active = tool_bar_data.active_index.clone();
    while let Some(message) = client.receive_message(ServerChannel::ToolBarMessage) {
        let tool_bar_message: ToolBarMessage = bincode::deserialize(&message).unwrap();
        match tool_bar_message {
            ToolBarMessage::SyncToolbar {
                index,
                staff_id,
                num,
            } => {
                if let Some(staff_id) = staff_id {
                    if let Some(staff) = staff_infos.get(staff_id) {
                        tool_bar_data.load_staff(index, staff, num);
                    }
                } else {
                    // 情况位置
                    tool_bar_data.empty_staff(index);
                }
            }
        }
        //重新激活方块
        tool_bar_data.active(active);
    }
}
