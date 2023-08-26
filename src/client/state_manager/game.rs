use std::marker::PhantomData;

use bevy::{
    input::mouse::MouseWheel,
    prelude::{
        in_state, AmbientLight, Commands, Entity, EventReader, Input, IntoSystemConfigs, KeyCode,
        Local, NextState, OnEnter, Plugin, Query, Res, ResMut, States, Update, Vec2, With,
    },
    window::{PrimaryWindow, Window},
};
use bevy_egui::{
    egui::{self, epaint::Shadow, Color32},
    EguiContext, EguiContexts, EguiUserTextures,
};
use bevy_renet::renet::{transport::NetcodeTransportError, RenetClient};
use renet_visualizer::{RenetClientVisualizer, RenetVisualizerStyle};

use crate::{
    client::{
        client_sync_players, client_sync_players_state,
        console_commands::ConsoleCommandPlugins,
        mesh_display::ClientMeshPlugin,
        player::{
            controller::{CharacterController, CharacterControllerPlugin},
            mouse_control::mouse_button_system,
            ClientLobby,
        },
        ray_cast::MeshRayCastPlugin,
        ui::{
            tool_bar::{tool_bar, ToolBar},
            UiPicResourceManager,
        },
    },
    common::ClientClipSpheresPlugin,
    sky::ClientSkyPlugins,
    CLIENT_DEBUG,
};

use super::{new_renet_client, ConnectionAddr, GameState};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum PlayState {
    Main,
    // 状态栏
    State,
    #[default]
    Disabled,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_state::<PlayState>();
        app.add_systems(OnEnter(GameState::Game), setup);
        if CLIENT_DEBUG {}
        app.insert_resource(RenetClientVisualizer::<200>::new(
            RenetVisualizerStyle::default(),
        ));
        app.add_systems(
            Update,
            update_visulizer_system.run_if(in_state(GameState::Game)),
        );
        app.add_systems(
            Update,
            (egui_center_cursor_system, mian_ui, controller_tool_bar)
                .run_if(in_state(PlayState::Main)),
        );
        // 这里是系统
        app.add_plugins(CharacterControllerPlugin);
        app.add_plugins(ClientClipSpheresPlugin::<CharacterController> { data: PhantomData });
        app.add_plugins(ClientMeshPlugin);
        app.add_plugins(ClientSkyPlugins);
        app.add_plugins(MeshRayCastPlugin);
        app.add_plugins(ConsoleCommandPlugins);

        app.add_systems(
            Update,
            (
                client_sync_players,
                client_sync_players_state,
                mouse_button_system,
                panic_on_error_system,
            )
                .run_if(bevy_renet::transport::client_connected())
                .run_if(in_state(GameState::Game)),
        );
    }
}

fn setup(
    mut commands: Commands,
    connection_addr: Res<ConnectionAddr>,
    mut play_state: ResMut<NextState<PlayState>>,
) {
    let (client, transport) = new_renet_client(connection_addr.clone());
    commands.insert_resource(client);
    commands.insert_resource(transport);
    commands.insert_resource(AmbientLight {
        brightness: 1.06,
        ..Default::default()
    });
    commands.insert_resource(ClientLobby::default());
    play_state.set(PlayState::Main);
}

fn update_visulizer_system(
    mut egui_contexts: EguiContexts,
    mut visualizer: ResMut<RenetClientVisualizer<200>>,
    client: Res<RenetClient>,
    mut show_visualizer: Local<bool>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    visualizer.add_network_info(client.network_info());
    if keyboard_input.just_pressed(KeyCode::F1) {
        *show_visualizer = !*show_visualizer;
    }
    if *show_visualizer {
        visualizer.show_window(egui_contexts.ctx_mut());
    }
}

// If any error is found we just panic
fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
    for e in renet_error.iter() {
        panic!("{}", e);
    }
}

// 中心十字

// 添加中心十字
pub fn egui_center_cursor_system(
    mut contexts: EguiContexts,
    window_qurey: Query<&mut Window, With<PrimaryWindow>>,
) {
    let ctx = contexts.ctx_mut();

    let Ok(window) = window_qurey.get_single() else{return;};
    let size = Vec2::new(window.width(), window.height());
    // 透明的屏幕！

    egui::CentralPanel::default()
        .frame(frame_transparent())
        .show(ctx, |ui| {
            // 计算十字准星的位置和大小
            let crosshair_size = 20.0;
            let crosshair_pos = egui::Pos2::new(
                size.x / 2.0 - crosshair_size / 2.0,
                size.y / 2.0 - crosshair_size / 2.0,
            );
            // 外边框
            let crosshair_rect =
                egui::Rect::from_min_size(crosshair_pos, egui::Vec2::splat(crosshair_size));

            // 绘制十字准星的竖线
            let line_width = 2.0;
            let line_rect = egui::Rect::from_min_max(
                egui::Pos2::new(
                    crosshair_rect.center().x - line_width / 2.0,
                    crosshair_rect.min.y,
                ),
                egui::Pos2::new(
                    crosshair_rect.center().x + line_width / 2.0,
                    crosshair_rect.max.y,
                ),
            );
            ui.painter()
                .rect_filled(line_rect, 1.0, egui::Color32::WHITE);

            // 绘制十字准星的横线
            let line_rect = egui::Rect::from_min_max(
                egui::Pos2::new(
                    crosshair_rect.min.x,
                    crosshair_rect.center().y - line_width / 2.0,
                ),
                egui::Pos2::new(
                    crosshair_rect.max.x,
                    crosshair_rect.center().y + line_width / 2.0,
                ),
            );
            ui.painter()
                .rect_filled(line_rect, 1.0, egui::Color32::WHITE);

            // todo 这里也可以添加下方物品栏
        });
}

fn mian_ui(
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
) {
    if let Ok((_, ctx, _)) = q.get_single_mut() {
        let bod_id = user_textures.image_id(&ui_pic_resource_manager.tool_box_border);
        egui::TopBottomPanel::bottom("tool_bar_bottom")
            .frame(frame_transparent())
            .resizable(false)
            .min_height(5.0)
            .show_separator_line(false)
            .show(ctx.into_inner().get_mut(), |ui| {
                ui.horizontal_centered(|ui| {
                    ui.vertical_centered_justified(|ui| {
                        tool_bar(
                            ui,
                            &mut tool_bar_data,
                            |image| user_textures.image_id(image),
                            bod_id.clone(),
                        );
                    });
                });
            });
    }
}

#[macro_export]
macro_rules! add_keyboard_toolbar {
    ($key: expr,$value: expr,$class: expr,$change:expr) => {
        if $class.just_pressed($key) {
            $change.active($value);
        }
    };
}

// 键盘控制 toolbar
fn controller_tool_bar(
    mut tool_bar_data: ResMut<ToolBar>,
    keyboard_input: Res<Input<KeyCode>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    for event in mouse_wheel_events.iter() {
        // println!("{:?}", event);
        let y = event.y;
        if y > 0. {
            tool_bar_data.active_next();
        } else if y < 0. {
            tool_bar_data.active_pre();
        }
    }
    add_keyboard_toolbar!(KeyCode::Key1, 0, keyboard_input, tool_bar_data);
    add_keyboard_toolbar!(KeyCode::Key2, 1, keyboard_input, tool_bar_data);
    add_keyboard_toolbar!(KeyCode::Key3, 2, keyboard_input, tool_bar_data);
    add_keyboard_toolbar!(KeyCode::Key4, 3, keyboard_input, tool_bar_data);
    add_keyboard_toolbar!(KeyCode::Key5, 4, keyboard_input, tool_bar_data);
    add_keyboard_toolbar!(KeyCode::Key6, 5, keyboard_input, tool_bar_data);
    add_keyboard_toolbar!(KeyCode::Key7, 6, keyboard_input, tool_bar_data);
    add_keyboard_toolbar!(KeyCode::Key8, 7, keyboard_input, tool_bar_data);
    add_keyboard_toolbar!(KeyCode::Key9, 8, keyboard_input, tool_bar_data);
    add_keyboard_toolbar!(KeyCode::Key0, 9, keyboard_input, tool_bar_data);

    if keyboard_input.just_pressed(KeyCode::Right) {
        tool_bar_data.active_next();
    }
    if keyboard_input.just_pressed(KeyCode::Left) {
        tool_bar_data.active_pre();
    }
}

fn frame_transparent() -> egui::containers::Frame {
    egui::containers::Frame {
        inner_margin: egui::style::Margin {
            left: 10.,
            right: 10.,
            top: 10.,
            bottom: 10.,
        },
        outer_margin: egui::style::Margin {
            left: 10.,
            right: 10.,
            top: 10.,
            bottom: 10.,
        },
        rounding: egui::Rounding {
            nw: 1.0,
            ne: 1.0,
            sw: 1.0,
            se: 1.0,
        },
        shadow: Shadow {
            extrusion: 1.0,
            color: Color32::TRANSPARENT,
        },
        fill: Color32::TRANSPARENT,
        stroke: egui::Stroke::new(2.0, Color32::TRANSPARENT),
        ..Default::default()
    }
}
