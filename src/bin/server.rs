use std::{net::UdpSocket, time::SystemTime};

use bevy::prelude::{
    App, Camera3dBundle, Commands, PointLightBundle, Res, ResMut, Startup, Transform, Update, Vec3,
};
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};
use bevy_renet::{
    renet::{
        transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
        RenetServer,
    },
    transport::NetcodeServerPlugin,
    RenetServerPlugin,
};
use just_join::{
    common::ServerClipSpheresPlugin,
    connection_config,
    server::{
        async_chunk::ChunkDataPlugin, chunk::ServerChunkPlugin,
        cross_through_check::CossTroughCheckPlugin, deal_message_system,
        object_filing::ObjectFilingPlugin, player::ServerLobby, server_connect_system,
        staff_rule_sync::ServerStaffRulePlugin, sync_body_and_head,
        terrain_physics::TerrainPhysicsPlugin,
    },
    sky::ServerSkyPlugins,
    staff::ServerStaffInfoPlugin,
    voxel_world::biomes::OtherTreePlugin,
    PROTOCOL_ID,
};
use renet_visualizer::RenetServerVisualizer;
use seldom_state::StateMachinePlugin;
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController},
    LookTransformPlugin,
};

#[cfg(feature = "server_ui")]
use {
    bevy::DefaultPlugins,
    bevy_egui::{EguiContexts, EguiPlugin},
    bevy_rapier3d::render::RapierDebugRenderPlugin,
    smooth_bevy_cameras::controllers::fps::FpsCameraPlugin,
};

#[cfg(feature = "headless")]
use bevy::MinimalPlugins;

fn new_renet_server() -> (RenetServer, NetcodeServerTransport) {
    let server = RenetServer::new(connection_config());

    let public_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(public_addr).unwrap();
    let current_time: std::time::Duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    // FIXME: 这里的写法 和master分支有出入 没有多态主机
    let server_config = ServerConfig {
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        authentication: ServerAuthentication::Unsecure,
        public_addr,
    };

    let transport = NetcodeServerTransport::new(current_time, server_config, socket).unwrap();

    (server, transport)
}

fn setup(mut commands: Commands) {
    // FIXME: 设置方便观察的相机 在必要时这个功能是不需要的
    // light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    commands
        .spawn(Camera3dBundle::default())
        .insert(FpsCameraBundle::new(
            FpsCameraController {
                // enabled: false,
                ..Default::default()
            },
            Vec3::new(-2.0, 60.0, 5.0),
            Vec3::new(0., 0., 0.),
            Vec3::Y,
        ));
}

fn main() {
    let mut app = App::new();

    #[cfg(feature = "server_ui")]
    {
        app.add_plugins(DefaultPlugins);
        app.add_plugins(RapierDebugRenderPlugin::default());
        app.add_plugins(EguiPlugin);
        app.add_plugins(FpsCameraPlugin::default());
    }

    #[cfg(feature = "headless")]
    {
        app.add_plugins(MinimalPlugins);
    }

    app.add_plugins(RenetServerPlugin);
    app.add_plugins(NetcodeServerPlugin);
    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
    app.add_plugins(LookTransformPlugin);

    // 这里添加必要的系统
    app.add_plugins((
        StateMachinePlugin,
        ServerStaffInfoPlugin,
        ServerClipSpheresPlugin,
        ServerChunkPlugin,
        TerrainPhysicsPlugin,
        ChunkDataPlugin,
        ServerSkyPlugins,
        ObjectFilingPlugin,
        ServerStaffRulePlugin,
        CossTroughCheckPlugin,
        OtherTreePlugin,
    ));

    let (server, transport) = new_renet_server();
    app.insert_resource(server);
    app.insert_resource(transport);
    app.insert_resource(RenetServerVisualizer::<200>::default());
    app.insert_resource(ServerLobby::default());

    app.add_systems(Startup, setup);
    app.add_systems(Update, update_visulizer_system);

    // TODO: 这里是必要的系统
    app.add_systems(
        Update,
        (
            server_connect_system,
            deal_message_system,
            sync_body_and_head,
        ),
    );
    app.run();
}

#[cfg(feature = "server_ui")]
fn update_visulizer_system(
    mut egui_contexts: EguiContexts,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
    server: Res<RenetServer>,
) {
    visualizer.update(&server);
    visualizer.show_window(egui_contexts.ctx_mut());
}

#[cfg(feature = "headless")]
fn update_visulizer_system(
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
    server: Res<RenetServer>,
) {
    visualizer.update(&server);
}
