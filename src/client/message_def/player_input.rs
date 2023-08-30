use bevy::prelude::{Component, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum PlayerInput {
    // 移动
    MOVE(Vec3),
    // 鼠标作用
    YAW(f32),
    PITCH(f32),
}
