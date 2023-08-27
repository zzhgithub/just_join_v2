use bevy::prelude::{Handle, Image, Resource};
use bevy_egui::egui;

use crate::staff::{Staff, StaffType};

use super::tool_box::tool_box;

#[derive(Debug, Clone)]
pub struct ToolBox {
    pub staff: Option<Staff>,
    pub num: usize,
    pub active: bool,
}

impl Default for ToolBox {
    fn default() -> Self {
        Self {
            staff: None,
            num: 0,
            active: false,
        }
    }
}

#[derive(Debug, Resource, Default, Clone)]
pub struct ToolBar {
    pub tools: [ToolBox; 10],
    pub active_index: usize,
}

impl ToolBar {
    // 加载物品
    pub fn load_staff(&mut self, index: usize, staff: Staff, num: usize) {
        self.tools[index % 10] = ToolBox {
            staff: Some(staff),
            num: num,
            ..Default::default()
        };
    }
    // 当前激活中的物品
    pub fn staff_type(&self) -> Option<StaffType> {
        if let Some(staff) = &self.tools[self.active_index].staff {
            Some(staff.staff_type.clone())
        } else {
            None
        }
    }
    pub fn active(&mut self, index: usize) {
        self.active_index = index;
        for i in 0..=9 {
            if i as usize == index {
                self.tools[i].active = true;
            } else {
                self.tools[i].active = false;
            }
        }
    }
    pub fn active_next(&mut self) {
        self.active_index += 1;
        if self.active_index > 9 {
            self.active(0);
        } else {
            self.active(self.active_index);
        }
    }
    pub fn active_pre(&mut self) {
        if self.active_index == 0 {
            self.active(9);
        } else {
            self.active_index -= 1;
            self.active(self.active_index);
        }
    }
}

pub fn tool_bar(
    ui: &mut egui::Ui,
    toolbar: &mut ToolBar,
    mut get_texture_egui: impl FnMut(&Handle<Image>) -> Option<egui::TextureId>,
    tool_box_border: Option<egui::TextureId>,
) {
    let mut rect = ui.available_rect_before_wrap();
    let ori_width = rect.width();
    let center_width = (64.0 + 2.) * 10. + 10. * 8.;
    rect.set_left(rect.left() + (ori_width - center_width) * 0.5);
    rect.set_right(rect.right() - (ori_width - center_width) * 0.5);
    rect.set_top(rect.bottom() - 50.0);
    ui.allocate_ui_at_rect(rect, |ui| {
        ui.horizontal(|ui| {
            for index in 0..=9 {
                let tool_box_data = &mut toolbar.tools[index as usize];
                let tool_box_item = tool_box(
                    ui,
                    &mut tool_box_data.active,
                    &mut tool_box_data.num,
                    if let Some(data) = tool_box_data.staff.clone() {
                        get_texture_egui(&data.icon.clone())
                    } else {
                        None
                    },
                    tool_box_border,
                );
                if tool_box_item.clicked() {
                    toolbar.active(index as usize);
                }
            }
        });
    });
}
