use bevy::prelude::{Input, KeyCode, Query, Res, ResMut};
use bevy_renet::renet::RenetClient;

use crate::client::{
    message_def::{user_command::UserCommandMessage, ClientChannel},
    ui::tool_bar::ToolBar,
};

use super::{controller::ControllerFlag, look::LookDirection};

pub fn deal_with_throw(
    keyboard_input: Res<Input<KeyCode>>,
    flags: Res<ControllerFlag>,
    tool_bar_data: Res<ToolBar>,
    mut client: ResMut<RenetClient>,
    query: Query<&LookDirection>,
) {
    if !flags.flag {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Q) {
        if let Some((index, staff)) = tool_bar_data.active_staff() {
            if let Ok(look) = query.get_single() {
                let message = bincode::serialize(&UserCommandMessage::Throw {
                    index: index,
                    staff_id: staff.id,
                    forward: look.forward,
                })
                .unwrap();
                client.send_message(ClientChannel::Command, message);
            }
        }
    }
}
