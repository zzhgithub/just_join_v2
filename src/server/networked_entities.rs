use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NetworkedEntities {
    pub client_ids: Vec<u64>,
    // 对象位移
    pub translations: Vec<[f32; 3]>,
    // 对象的头部动作
    pub yaws: Vec<f32>,
    pub pitch: Vec<f32>,
}
