use bevy::prelude::{
    AssetServer, Commands, Handle, Image, IntoSystemConfigs, Plugin, Res, Resource, Startup,
};
use bevy_egui::EguiContexts;

use crate::staff::{StaffInfoStroge, StaffSet};

// 这里是尝试管理自定义UI的
pub mod staff_rules;
pub mod test;
pub mod tool_bar;
pub mod tool_box;

// 加载staff到egui
// 加载UI需要的资源文件

#[derive(Debug, Resource, Default)]
pub struct UiPicResourceManager {
    pub tool_box_border: Handle<Image>,
}

pub struct UiResourcePlugin;

impl Plugin for UiResourcePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, init_egui_resource.after(StaffSet::Init));
    }
}

fn init_egui_resource(
    mut commands: Commands,
    staff_info_stroge: Res<StaffInfoStroge>,
    mut contexts: EguiContexts,
    asset_server: Res<AssetServer>,
) {
    for (_, staff) in staff_info_stroge.data.iter() {
        contexts.add_image(staff.icon.clone());
    }

    let tool_box_border: Handle<Image> = asset_server.load("ui/item_slot.png");
    contexts.add_image(tool_box_border.clone());

    commands.insert_resource(UiPicResourceManager { tool_box_border })
}
