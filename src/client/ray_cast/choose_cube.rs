use bevy::prelude::{Resource, Vec3, Component};

#[derive(Resource, Debug, Clone, Copy)]
pub struct ChooseCube {
    // 选中的点
    pub choose_on: Option<Vec3>,
    // 选择中的点对应的方块
    pub center: Option<Vec3>,
    // 选中点 法向量对面的方块
    pub out_center: Option<Vec3>,
}

impl ChooseCube {
    pub fn new() -> Self {
        Self {
            choose_on: None,
            center: None,
            out_center: None,
        }
    }
}


#[derive(Component)]
pub struct HelpCube;