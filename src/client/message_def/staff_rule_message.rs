use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Component)]
pub struct StaffRuleMessage {
    pub staff_rule_id: u32,
    pub need: Vec<(usize, usize, usize)>,
    pub times: usize,
}
