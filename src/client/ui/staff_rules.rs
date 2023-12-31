// 合成相关UI
use bevy::{
    prelude::{Entity, Query, Res, ResMut, Resource, With},
    window::{PrimaryWindow, Window},
};
use bevy_easy_localize::Localize;
use bevy_egui::{
    egui::{self, Align2, Id, Vec2},
    EguiContext, EguiUserTextures,
};
use bevy_renet::renet::RenetClient;
use egui_extras::{Column, Size, StripBuilder, TableBuilder};

use crate::{
    client::message_def::{staff_rule_message::StaffRuleMessage, ClientChannel},
    staff::{
        rule::{StaffRule, StaffRules},
        StaffInfoStroge,
    },
};

use super::tool_bar::ToolBar;

#[derive(Debug, Resource)]
pub struct MyMemory(pub egui::Memory);

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
    // ui_pic_resource_manager: Res<UiPicResourceManager>,
    tool_bar_data: Res<ToolBar>,
    staff_rules: Res<StaffRules>,
    staff_info_stroge: Res<StaffInfoStroge>,
    mut client: ResMut<RenetClient>,
    localize: Res<Localize>,
    mut memory: ResMut<MyMemory>,
) {
    // 这里显示合成列表
    if let Ok((_, ctx, _)) = q.get_single_mut() {
        let ctx = ctx.into_inner().get_mut();
        let windows = egui::Window::new(localize.get("合成列表"))
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
                                    .resizable(true)
                                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                    .column(Column::auto())
                                    .column(Column::auto())
                                    .column(Column::initial(100.).range(100.0..=300.))
                                    .column(Column::remainder())
                                    .min_scrolled_height(0.0);

                                table
                                    .header(20.0, |mut header| {
                                        header.col(|ui| {
                                            ui.strong(localize.get("依赖"));
                                        });
                                        header.col(|ui| {
                                            ui.strong(localize.get("规则"));
                                        });
                                        header.col(|ui| {
                                            ui.strong(localize.get("输出"));
                                        });
                                        header.col(|ui| {
                                            ui.strong(localize.get("操作"));
                                        });
                                    })
                                    .body(|mut body| {
                                        for (_, ele) in staff_rules.rules.clone() {
                                            let staff_rule = ele.clone();
                                            body.row(100., |mut row| {
                                                row.col(|ui| {
                                                    if let Some(e) = staff_rule.base_on {
                                                        ui.label(format!("{}", e));
                                                    } else {
                                                        ui.label("无");
                                                    }
                                                });
                                                row.col(|ui| {
                                                    for pair in staff_rule.input {
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
                                                    for pair in staff_rule.output {
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
                                                                if pair.num_needed > 1 {
                                                                    ui.label(format!(
                                                                        "x {}",
                                                                        pair.num_needed
                                                                    ));
                                                                }
                                                            }
                                                        }
                                                    }
                                                });
                                                row.col(|ui| {
                                                    // 这里数字框
                                                    let num = memory
                                                        .0
                                                        .data
                                                        .get_temp_mut_or(Id::new(ele.id), 1);

                                                    if ui.button("-").clicked() && *num > 1 {
                                                        *num -= 1;
                                                    }
                                                    ui.label(format!("{}", num));
                                                    if ui.button("+").clicked() && *num < 999 {
                                                        *num += 1;
                                                    }
                                                    if let Some(needed) = can_make_by_staff(
                                                        ele.clone(),
                                                        &tool_bar_data,
                                                        num.clone(),
                                                    ) {
                                                        if ui.button("合成").clicked() {
                                                            // 判断使用按钮
                                                            println!("点击了合成按钮");
                                                            let message = bincode::serialize(
                                                                &StaffRuleMessage {
                                                                    staff_rule_id: ele.id.clone(),
                                                                    need: needed,
                                                                    times: num.clone(),
                                                                },
                                                            )
                                                            .unwrap();
                                                            client.send_message(
                                                                ClientChannel::StaffRule,
                                                                message,
                                                            );
                                                        }
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

fn can_make_by_staff(
    staff_rule: StaffRule<u32>,
    toolbar: &ToolBar,
    num: usize,
) -> Option<Vec<(usize, usize, usize)>> {
    // 需要的
    let mut needed: Vec<(usize, usize, usize)> = Vec::new();
    for pair in staff_rule.input {
        if let Some(rs) = toolbar.need_staff(pair.staff_id, pair.num_needed * num) {
            needed.append(&mut rs.clone());
        } else {
            return None;
        }
    }
    return Some(needed);
}
