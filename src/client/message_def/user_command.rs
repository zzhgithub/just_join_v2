use bevy::prelude::{Component, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum UserCommandMessage {
    // 丢弃物品
    Throw {
        index: usize,
        staff_id: usize,
        forward: Vec3,
    },
}
