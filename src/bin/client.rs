use std::{marker::PhantomData, net::UdpSocket, time::SystemTime};

use bevy::{
    prelude::{
        AmbientLight, App, EventReader, Input, IntoSystemConfigs, KeyCode, Local, Res, ResMut,
        Update,
    },
    DefaultPlugins,
};
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_renet::{
    renet::{
        transport::{ClientAuthentication, NetcodeClientTransport, NetcodeTransportError},
        RenetClient,
    },
    transport::NetcodeClientPlugin,
    RenetClientPlugin,
};
use just_join::{
    client::{
        client_sync_players, client_sync_players_state,
        mesh_display::ClientMeshPlugin,
        player::{
            controller::{CharacterController, CharacterControllerPlugin},
            ClientLobby,
        },
    },
    common::{ClientClipSpheresPlugin, ClipSpheres},
    connection_config,
    sky::ClientSkyPlugins,
    tools::inspector_egui::inspector_ui,
    PROTOCOL_ID,
};
use renet_visualizer::{RenetClientVisualizer, RenetVisualizerStyle};

// 创建连接
fn new_renet_client() -> (RenetClient, NetcodeClientTransport) {
    let client = RenetClient::new(connection_config());

    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    // 这里为了生成唯一的id
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    (client, transport)
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(RenetClientPlugin);
    app.add_plugins(NetcodeClientPlugin);
    app.add_plugins(EguiPlugin);

    // 游戏相关系统
    app.add_plugins(CharacterControllerPlugin);
    app.add_plugins(ClientClipSpheresPlugin::<CharacterController> { data: PhantomData });
    app.add_plugins(ClientMeshPlugin);
    app.add_plugins(ClientSkyPlugins);

    let (client, transport) = new_renet_client();
    app.insert_resource(client);
    app.insert_resource(transport);
    app.insert_resource(RenetClientVisualizer::<200>::new(
        RenetVisualizerStyle::default(),
    ));
    // 设置一个环境光照强度
    app.insert_resource(AmbientLight {
        brightness: 1.06,
        ..Default::default()
    });
    app.insert_resource(ClientLobby::default());
    // 调试工具
    app.add_systems(Update, inspector_ui);
    app.add_plugins(ResourceInspectorPlugin::<ClipSpheres>::default());
    app.register_type::<ClipSpheres>();
    // TODO: 其他系统
    app.add_systems(
        Update,
        (client_sync_players, client_sync_players_state)
            .run_if(bevy_renet::transport::client_connected()),
    );

    // 这只负责展示 player 和 mesh的展示! 没有本地的物理引擎
    app.add_systems(Update, update_visulizer_system);
    app.add_systems(Update, panic_on_error_system);
    app.run();
}

// If any error is found we just panic
fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
    for e in renet_error.iter() {
        panic!("{}", e);
    }
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
