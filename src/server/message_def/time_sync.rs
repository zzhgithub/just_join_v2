use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum TimeSync {
    SkyBox(f32),
}
