use std::marker::PhantomData;

use bevy::prelude::{
    in_state, AmbientLight, Commands, EventReader, Input, IntoSystemConfigs, KeyCode, Local,
    NextState, OnEnter, Plugin, Res, ResMut, States, Update,
};
use bevy_egui::EguiContexts;
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
