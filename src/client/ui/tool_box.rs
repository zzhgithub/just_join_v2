use bevy_egui::egui::{self, Align2, Color32, FontId, Rect, Stroke};

/// 自定义组件 工具盒子

pub fn tool_box(
    ui: &mut egui::Ui,
    on: &mut bool,
    num: &mut usize,
    texture_id: Option<egui::TextureId>,
    tool_box_border: Option<egui::TextureId>,
) -> egui::Response {
    // 这里的大小是默认的
    let desired_size = egui::vec2(64.0, 64.0);
    //分配空间
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    // 点击事件
    if response.clicked() {
        // *on = !*on;
        response.mark_changed();
    }

    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

    // 如果足够分配空间
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);

        ui.painter().rect(
            rect,
            2.,
            Color32::TRANSPARENT,
            if *on {
                Stroke {
                    width: visuals.bg_stroke.width,
                    color: Color32::RED,
                }
            } else {
                visuals.bg_stroke
            },
        );

        if let Some(txt_id) = tool_box_border {
            let rect_clone = rect;
            ui.painter().image(
                txt_id,
                rect_clone,
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        // 绘制图片如果有图片的话
        if *num > 0 {
            if let Some(text_id) = texture_id {
                let rect_clone = rect;
                ui.painter().image(
                    text_id,
                    rect_clone.expand(-10.0),
                    Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            let mut pos = rect.max;
            pos.x -= 32.;
            pos.y -= 4.;
            ui.painter().text(
                pos,
                Align2::CENTER_CENTER,
                format!("x{}", num),
                FontId::new(10.0, egui::FontFamily::Monospace),
                Color32::WHITE,
            );
        }
    }
    if *on {
        response.highlight()
    } else {
        response
    }
}
