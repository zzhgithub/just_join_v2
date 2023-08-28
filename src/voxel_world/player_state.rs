use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

use super::map_database::MapDataBase;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PlayerState {
    pub position: [f32; 3],
    pub toolbar: [(Option<usize>, usize); 10],
}

impl PlayerState {}

pub trait StorgePlayerState {
    fn save_player_state(
        &mut self,
        username: String,
        player_state: PlayerState,
    ) -> Option<PlayerState>;
    fn get_player_state(&self, username: String) -> Option<PlayerState>;
}

impl StorgePlayerState for MapDataBase {
    fn save_player_state(
        &mut self,
        username: String,
        player_state: PlayerState,
    ) -> Option<PlayerState> {
        let key_str = format!("U:{}", username);
        let key = key_str.as_bytes();
        match self
            .db
            .insert(key, bincode::serialize(&player_state.clone()).unwrap())
        {
            Ok(_) => Some(player_state),
            Err(_) => {
                println!("保存玩家数据时出错");
                None
            }
        }
    }
    fn get_player_state(&self, username: String) -> Option<PlayerState> {
        let key_str = format!("U:{}", username);
        let key = key_str.as_bytes();
        match self.db.get(key) {
            Ok(rs) => rs.map(|data| bincode::deserialize(&data).unwrap()),
            Err(_) => {
                println!("获取玩家状态时报错");
                None
            }
        }
    }
}

#[derive(Debug, Component, Clone)]
pub struct PlayerOntimeState(pub PlayerState);
