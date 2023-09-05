use std::time::Duration;

use bevy::{
    asset::ChangeWatcher,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::{App, AssetPlugin, PluginGroup, ResMut, Startup, Update},
    window::{ExitCondition, WindowPlugin},
    DefaultPlugins,
};
use bevy_easy_localize::{Localize, LocalizePlugin};
use bevy_egui::EguiPlugin;
use bevy_mod_billboard::prelude::BillboardPlugin;
use bevy_renet::{transport::NetcodeClientPlugin, RenetClientPlugin};
use bevy_sprite3d::Sprite3dPlugin;
use just_join::{
    client::{
        debug::ClientDebugPlugin,
        state_manager::{
            game::GamePlugin, menu::MenuPlugin, notification::NotificationPlugin,
            splash::SplashPlugin, ConnectionAddr, GameState, CHINESE,
        },
        ui::UiResourcePlugin,
    },
    staff::StaffInfoPlugin,
    tools::inspector_egui::inspector_ui,
    CLIENT_DEBUG, CLIENT_FPS,
};
fn main() {
    let mut app = App::new();
    app.add_plugins(WindowPlugin {
        exit_condition: ExitCondition::OnAllClosed,
        close_when_requested: false,
        ..Default::default()
    });
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                ..Default::default()
            })
            .build()
            .disable::<WindowPlugin>(),
    );
    app.add_state::<GameState>();
    app.insert_resource(ConnectionAddr::default());
    app.add_plugins(LocalizePlugin);
    app.insert_resource(Localize::from_asset_path("translation.csv"));
    app.add_plugins(BillboardPlugin);
    app.add_plugins(RenetClientPlugin);
    app.add_plugins(NetcodeClientPlugin);
    app.add_plugins(EguiPlugin);
    app.add_plugins(StaffInfoPlugin);
    app.add_plugins(UiResourcePlugin);
    app.add_plugins(Sprite3dPlugin);

    app.add_plugins((SplashPlugin, MenuPlugin, NotificationPlugin, GamePlugin));
    // 调试工具
    if CLIENT_DEBUG {
        app.add_systems(Update, inspector_ui);
        app.add_plugins(ClientDebugPlugin);
    }
    if CLIENT_FPS {
        app.add_plugins((
            // Adds frame time diagnostics
            FrameTimeDiagnosticsPlugin,
            // Adds a system that prints diagnostics to the console
            LogDiagnosticsPlugin::default(),
        ));
    }
    app.add_systems(Startup, setting_language);
    app.run();
}

//setting of switch the lanuguage
fn setting_language(mut localize: ResMut<Localize>) {
    localize.set_language(CHINESE);
    println!("开始是:{}", localize.get("开始"));
}
