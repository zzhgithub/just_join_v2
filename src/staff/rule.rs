//这里表示合成的公式

use bevy::{
    prelude::{error, Plugin, ResMut, Resource, Startup},
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffNumPair {
    pub staff_id: usize,
    pub num_needed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffRule<T> {
    pub id: u32,
    // 输入要求
    pub input: Vec<StaffNumPair>,
    // 输出类型
    pub output_id: usize,
    // 需要的依赖
    pub base_on: Option<T>,
    // 描述
    pub desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct StaffRules {
    pub rules: HashMap<u32, StaffRule<u32>>,
}

// 加载这里的数据

pub struct StaffRulePlugin;

impl Plugin for StaffRulePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(StaffRules {
            rules: HashMap::new(),
        });
        app.add_systems(Startup, setup);
    }
}

fn setup(mut staff_rules: ResMut<StaffRules>) {
    let path = "staff_rules.ron";
    match std::fs::File::open(path) {
        Ok(file) => {
            let res: Vec<StaffRule<u32>> = ron::de::from_reader(file).unwrap();
            for ele in res {
                staff_rules.rules.insert(ele.id.clone(), ele.clone());
            }
        }
        Err(_) => {
            error!("合成规则表获取失败");
        }
    }
}
