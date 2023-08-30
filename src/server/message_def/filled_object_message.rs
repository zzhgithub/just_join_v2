use bevy::prelude::{Component, Entity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum FilledObjectMessage {
    // 同步区块内掉落物
    SyncFilledObject(Vec<(Entity, usize, [f32; 3])>),
}
