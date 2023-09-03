// 合成相关UI

use bevy::{
    prelude::{Entity, Query, Res, ResMut, With},
    window::{PrimaryWindow, Window},
};
use bevy_egui::{
    egui::{self, Align2, Vec2},
    EguiContext, EguiUserTextures,
};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};

use crate::staff::{rule::StaffRules, StaffInfoStroge};

use super::{tool_bar::ToolBar, UiPicResourceManager};

pub fn staff_rules_ui(
    mut q: Query<
        (
            Entity,
            &'static mut EguiContext,
            Option<&'static PrimaryWindow>,
        ),
        With<Window>,
    >,
    user_textures: Res<EguiUserTextures>,
    ui_pic_resource_manager: Res<UiPicResourceManager>,
    mut tool_bar_data: ResMut<ToolBar>,
    staff_rules: Res<StaffRules>,
    staff_info_stroge: Res<StaffInfoStroge>,
) {
    // 这里显示合成列表
    if let Ok((_, ctx, _)) = q.get_single_mut() {
        let ctx = ctx.into_inner().get_mut();
        let windows = egui::Window::new("合成列表")
            .id(egui::Id::new("staff Rules"))
            .fixed_size(Vec2::new(800., 600.))
            .resizable(false)
            .collapsible(false)
            .title_bar(true)
            .scroll2([false, false])
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO);
        windows.show(ctx, |ui| {
            ui.vertical(|ui| {
                // TODO 先展示 物品栏和合计的物品个数？
                StripBuilder::new(ui)
                    .size(Size::remainder().at_least(100.0)) // for the table
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            egui::ScrollArea::horizontal().show(ui, |ui| {
                                let table = TableBuilder::new(ui)
                                    .striped(true)
                                    .resizable(false)
                                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                    .column(Column::auto())
                                    .column(Column::auto())
                                    .column(Column::initial(100.).range(100.0..=300.))
                                    .column(Column::remainder())
                                    .min_scrolled_height(0.0);

                                table
                                    .header(20.0, |mut header| {
                                        header.col(|ui| {
                                            ui.strong("Base On");
                                        });
                                        header.col(|ui| {
                                            ui.strong("Rule");
                                        });
                                        header.col(|ui| {
                                            ui.strong("Output");
                                        });
                                        header.col(|ui| {
                                            ui.strong("Action");
                                        });
                                    })
                                    .body(|mut body| {
                                        for ele in staff_rules.rules.clone() {
                                            body.row(100., |mut row| {
                                                row.col(|ui| {
                                                    if let Some(e) = ele.base_on {
                                                        ui.label(format!("{}", e));
                                                    } else {
                                                        ui.label("无");
                                                    }
                                                });
                                                row.col(|ui| {
                                                    for pair in ele.input {
                                                        if let Some(staff) =
                                                            staff_info_stroge.get(pair.staff_id)
                                                        {
                                                            if let Some(txt_id) =
                                                                user_textures.image_id(&staff.icon)
                                                            {
                                                                ui.image(
                                                                    txt_id,
                                                                    Vec2::new(64., 64.),
                                                                );
                                                                ui.label(format!(
                                                                    "x {}",
                                                                    pair.num_needed
                                                                ));
                                                            }
                                                        }
                                                    }
                                                });
                                                row.col(|ui| {
                                                    if let Some(staff) =
                                                        staff_info_stroge.get(ele.output_id)
                                                    {
                                                        if let Some(txt_id) =
                                                            user_textures.image_id(&staff.icon)
                                                        {
                                                            ui.image(txt_id, Vec2::new(64., 64.));
                                                        }
                                                    }
                                                });
                                                row.col(|ui| {
                                                    if ui.button("合成").clicked() {
                                                        // TODO 判断使用按钮
                                                        println!("点击了合成按钮")
                                                    }
                                                });
                                            });
                                        }
                                    });
                            });
                        });
                    });
            });
        });
    }
}
