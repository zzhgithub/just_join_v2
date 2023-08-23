use bevy::prelude::{
    in_state, App, AssetServer, Input, IntoSystemConfigs, MouseButton, NextState, OnEnter, Plugin,
    Res, ResMut, Update,
};
use bevy_egui::{
    egui::{self, FontDefinitions, FontId, RichText, TextStyle},
    EguiContexts,
};

use super::GameState;
pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        // As this plugin is managing the splash screen, it will focus on the state `GameState::Splash`
        app.add_systems(OnEnter(GameState::Splash), splash_setup)
            .add_systems(
                Update,
                (show, countdown).run_if(in_state(GameState::Splash)),
            );
    }
}

fn splash_setup(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();
    let mut fonts = FontDefinitions::default();
    // 设置外部字体
    fonts.font_data.insert(
        "ark_font".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../../assets/font/fusion-pixel-12px-monospaced-zh_hans.ttf"
        )),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "ark_font".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("ark_font".to_owned());
    ctx.set_fonts(fonts);
    // 自定义字体型号

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(40.0, egui::FontFamily::Proportional),
        ),
        (heading2(), FontId::new(22.0, egui::FontFamily::Monospace)),
        (
            heading3(),
            FontId::new(19.0, egui::FontFamily::Proportional),
        ),
        (
            TextStyle::Body,
            FontId::new(16.0, egui::FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(12.0, egui::FontFamily::Monospace),
        ),
        (
            TextStyle::Button,
            FontId::new(22.0, egui::FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(8.0, egui::FontFamily::Proportional),
        ),
    ]
    .into();
    ctx.set_style(style);
}

fn show(mut contexts: EguiContexts, asset_server: Res<AssetServer>) {
    let image = asset_server.load("branding/icon.png");
    let text_id = contexts.add_image(image);
    let ctx = contexts.ctx_mut();
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal_centered(|ui| {
            ui.vertical_centered_justified(|ui| {
                ui.allocate_ui_at_rect(ui.available_rect_before_wrap().shrink(100.), |_ui| {});
                ui.image(text_id, [200., 200.]);
                ui.label("Power by Bevy.");
                // 插入距离
                ui.allocate_ui_at_rect(ui.available_rect_before_wrap().shrink(20.), |_ui| {});
                ui.label("Created By:");
                ui.label(
                    RichText::new("重庆星空液化委员会.")
                        .text_style(heading2())
                        .strong(),
                )
            });
        });
    });
}
// Tick the timer, and change state when finished
fn countdown(
    mut game_state: ResMut<NextState<GameState>>,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        game_state.set(GameState::Menu);
    }
}

#[inline]
fn heading2() -> TextStyle {
    TextStyle::Name("Heading2".into())
}

#[inline]
fn heading3() -> TextStyle {
    TextStyle::Name("ContextHeading".into())
}
