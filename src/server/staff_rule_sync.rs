use bevy::prelude::{Entity, EventWriter, Plugin, Query, Res, ResMut, Transform, Update};
use bevy_renet::renet::RenetServer;

use crate::{
    client::message_def::{staff_rule_message::StaffRuleMessage, ClientChannel},
    staff::{
        rule::{StaffNumPair, StaffRules},
        StaffInfoStroge,
    },
    tools::vec3_to_chunk_key_any_xyz,
    voxel_world::player_state::PlayerOntimeState,
};

use super::{
    object_filing::ObjectFillEvent, player::ServerLobby, tool_bar_sync::send_all_tool_bar,
};

pub struct ServerStaffRulePlugin;

impl Plugin for ServerStaffRulePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, deal_with_staff_rule);
    }
}

pub fn deal_with_staff_rule(
    mut server: ResMut<RenetServer>,
    lobby: ResMut<ServerLobby>,
    mut query: Query<(Entity, &Transform, &mut PlayerOntimeState)>,
    staff_rules: Res<StaffRules>,
    mut fill_event: EventWriter<ObjectFillEvent>,
    staff_info_stroge: Res<StaffInfoStroge>,
) {
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::StaffRule) {
            let StaffRuleMessage {
                staff_rule_id,
                need,
                times,
            } = bincode::deserialize(&message).unwrap();
            if let Some(rule) = staff_rules.rules.get(&staff_rule_id) {
                if let Some(entity) = lobby.players.get(&client_id) {
                    if let Ok((_, trf, mut player_state)) = query.get_mut(*entity) {
                        // 找到用户的Ontime 减少物品
                        for (index, staff_id, use_num) in need {
                            if player_state.0.use_staff(index, staff_id, use_num) == None {
                                println!("物体生成时消耗错误！");
                                println!("{}|{}|{}", index, staff_id, use_num);
                            }
                        }
                        for StaffNumPair {
                            staff_id,
                            num_needed,
                        } in rule.output.clone()
                        {
                            if let Some(out_staff) = staff_info_stroge.get(staff_id) {
                                for _ in 0..num_needed * times {
                                    let center = trf.translation;
                                    let (chunk_key, xyz) = vec3_to_chunk_key_any_xyz(center);
                                    // 生成新的掉落物
                                    fill_event.send(ObjectFillEvent {
                                        chunk_key: chunk_key,
                                        xyz: xyz,
                                        center: center,
                                        staff: out_staff.clone(),
                                    });
                                }
                            }
                        }
                        // 数据都处理完了 再一起同步
                        send_all_tool_bar(client_id, &mut server, player_state.0.clone());
                    }
                }
            }
        }
    }
}
