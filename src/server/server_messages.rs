use bevy::prelude::{Component, Entity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessages {
    // 创建角色
    PlayerCreate {
        entity: Entity,
        id: u64,
        translation: [f32; 3],
    },
    // 删除角色
    PlayerRemove {
        id: u64,
    },
}
