use bevy::{
    app::AppExit,
    prelude::{
        in_state, not, Entity, EventReader, EventWriter, IntoSystemConfigs, NextState, OnEnter,
        Plugin, Query, Res, ResMut, Resource, States, Update, With,
    },
    window::{PrimaryWindow, Window, WindowCloseRequested},
};
use bevy_easy_localize::Localize;
use bevy_egui::{egui, EguiContext, EguiContexts, EguiUserTextures};
use std::time::Duration;

use super::{notification::Notification, ConnectionAddr, GameState};
use super::{CHINESE, ENGLISH};
use crate::{
    client::{
        player::controller::back_grab_cursor,
        ui::{
            test::toggle_ui,
            tool_bar::{tool_bar, ToolBar},
            tool_box::tool_box,
            UiPicResourceManager,
        },
    },
    staff::StaffInfoStroge,
    tools::string::{is_port, is_valid_server_address},
    CLIENT_DEBUG,
};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum MenuState {
    Main,
    Test,
    Settings,
    Multiplayer,
    #[default]
    Disabled,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_state::<MenuState>();
        app.insert_resource(TestResource::default());
        app.insert_resource(ToolBar::default());
        app.add_systems(OnEnter(GameState::Menu), (setup, back_grab_cursor));
        app.add_systems(Update, menu_main.run_if(in_state(MenuState::Main)));
        app.add_systems(Update, test.run_if(in_state(MenuState::Test)));
        app.add_systems(Update, menu_settings.run_if(in_state(MenuState::Settings)));
        app.add_systems(
            Update,
            menu_multiplayer.run_if(in_state(MenuState::Multiplayer)),
        );

        app.add_systems(
            Update,
            disconnect_on_close_without_connected.run_if(not(in_state(GameState::Game))),
        );
    }
}

fn menu_settings(
    mut localize: ResMut<Localize>,
    mut contexts: EguiContexts,
    mut menu_state: ResMut<NextState<MenuState>>,
) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.heading(localize.get("设置"));
        if ui.button(localize.get("切换英语")).clicked() {
            localize.set_language(ENGLISH);
        }
        if ui.button(localize.get("切换中文")).clicked() {
            localize.set_language(CHINESE);
        }
        if ui.button(localize.get("返回")).clicked() {
            // 状态转移到 多人游戏的设置
            menu_state.set(MenuState::Main);
        }
    });
}

fn menu_multiplayer(
    localize: Res<Localize>,
    mut contexts: EguiContexts,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut connection_addr: ResMut<ConnectionAddr>,
    mut notification: ResMut<Notification>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let ctx = contexts.ctx_mut();
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(localize.get("多人游戏"));
        ui.label(localize.get("服务器"));
        ui.text_edit_singleline(&mut connection_addr.server);

        ui.label(localize.get("端口"));
        ui.text_edit_singleline(&mut connection_addr.port);

        ui.label(localize.get("昵称"));
        ui.text_edit_singleline(&mut connection_addr.nickname);
        if ui.button(localize.get("开始")).clicked() {
            // 这开始游戏相关数据
            if connection_addr.server.is_empty() {
                notification
                    .toasts
                    .error("Server 不是为空")
                    .set_duration(Some(Duration::from_secs(5)));
            } else if !is_valid_server_address(connection_addr.server.as_str()) {
                // 判断数据是否合法
                notification
                    .toasts
                    .error("Server 不是ip地址")
                    .set_duration(Some(Duration::from_secs(5)));
            } else if !is_port(connection_addr.port.as_str()) {
                notification
                    .toasts
                    .error("Port 不是数字")
                    .set_duration(Some(Duration::from_secs(5)));
            } else if connection_addr.port.is_empty() {
                notification
                    .toasts
                    .error("Port 为空")
                    .set_duration(Some(Duration::from_secs(5)));
            } else {
                notification
                    .toasts
                    .info(localize.get("进入服务器"))
                    .set_duration(Some(Duration::from_secs(5)));
                menu_state.set(MenuState::Disabled);
                game_state.set(GameState::Game);
            }
        }
        if ui.button(localize.get("返回")).clicked() {
            // 状态转移到 多人游戏的设置
            menu_state.set(MenuState::Main);
        }
    });
}

// 游戏主界面
fn menu_main(
    localize: Res<Localize>,
    mut contexts: EguiContexts,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.heading("Welcome to Just Join!");
        // ui.image(texture_id, size)
        if ui.button(localize.get("多人游戏")).clicked() {
            // 状态转移到 多人游戏的设置
            menu_state.set(MenuState::Multiplayer)
        }
        if ui.button(localize.get("设置")).clicked() {
            // 转到设计游戏的地方
            menu_state.set(MenuState::Settings);
        }
        if ui.button(localize.get("退出")).clicked() {
            // 退出游戏
            app_exit_events.send(AppExit);
        }
        if ui.button(localize.get("关于")).clicked() {
            // 显示游戏制作人和版本号
            menu_state.set(MenuState::Disabled);
            game_state.set(GameState::Splash);
        }

        if CLIENT_DEBUG && ui.button("测试").clicked() {
            menu_state.set(MenuState::Test);
        }
    });
}

fn setup(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

#[derive(Debug, Resource, Default)]
pub struct TestResource {
    pub flag: bool,
}

fn test(
    // mut contexts: EguiContexts,
    mut q: Query<
        (
            Entity,
            &'static mut EguiContext,
            Option<&'static PrimaryWindow>,
        ),
        With<Window>,
    >,
    user_textures: Res<EguiUserTextures>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut test_resource: ResMut<TestResource>,
    staff_info_stroge: Res<StaffInfoStroge>,
    ui_pic_resource_manager: Res<UiPicResourceManager>,
    mut tool_bar_data: ResMut<ToolBar>,
) {
    if let Ok((_, ctx, _)) = q.get_single_mut() {
        if let Some((_, staff)) = staff_info_stroge.data.iter().next() {
            let text_id = user_textures.image_id(&staff.icon.clone());
            let bod_id = user_textures.image_id(&ui_pic_resource_manager.tool_box_border);
            egui::CentralPanel::default().show(ctx.into_inner().get_mut(), |ui| {
                ui.heading("Testing:");
                let mut num: usize = 999;
                // test start
                toggle_ui(ui, &mut test_resource.flag);
                tool_box(ui, &mut test_resource.flag, &mut num, text_id, bod_id);
                tool_bar(
                    ui,
                    &mut tool_bar_data,
                    |image| user_textures.image_id(image),
                    bod_id,
                );
                // test end
                if ui.button("Back").clicked() {
                    // 状态转移到 多人游戏的设置
                    menu_state.set(MenuState::Main);
                }
            });
        }
    }
}

fn disconnect_on_close_without_connected(
    mut exit: EventWriter<AppExit>,
    mut closed: EventReader<WindowCloseRequested>,
) {
    for _ in closed.iter() {
        exit.send(AppExit);
    }
}
