use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ToolBarMessage {
    SyncToolbar {
        index: usize,
        staff_id: Option<usize>,
        num: usize,
    },
}
