use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::{App, Update},
    DefaultPlugins,
};
use bevy_egui::EguiPlugin;
use bevy_mod_billboard::prelude::BillboardPlugin;
use bevy_renet::{transport::NetcodeClientPlugin, RenetClientPlugin};
use just_join::{
    client::{
        state_manager::{
            game::GamePlugin, menu::MenuPlugin, notification::NotificationPlugin,
            splash::SplashPlugin, ConnectionAddr, GameState,
        },
        ui::UiResourcePlugin,
    },
    staff::StaffInfoPlugin,
    tools::inspector_egui::inspector_ui,
    CLIENT_DEBUG,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_state::<GameState>();
    app.insert_resource(ConnectionAddr::default());

    app.add_plugins(BillboardPlugin);
    app.add_plugins(RenetClientPlugin);
    app.add_plugins(NetcodeClientPlugin);
    app.add_plugins(EguiPlugin);
    app.add_plugins(StaffInfoPlugin);
    app.add_plugins(UiResourcePlugin);

    app.add_plugins((SplashPlugin, MenuPlugin, NotificationPlugin, GamePlugin));
    // 调试工具
    if CLIENT_DEBUG {
        app.add_plugins((
            // Adds frame time diagnostics
            FrameTimeDiagnosticsPlugin,
            // Adds a system that prints diagnostics to the console
            LogDiagnosticsPlugin::default(),
        ));
        app.add_systems(Update, inspector_ui);
    }
    app.run();
}
