use std::{net::UdpSocket, time::SystemTime};

use bevy::prelude::{
    Camera2dBundle, Commands, Component, DespawnRecursiveExt, Entity, Query, Resource, States, With,
};
use bevy_renet::renet::{
    transport::{ClientAuthentication, NetcodeClientTransport},
    RenetClient,
};

use crate::{connection_config, users::Username, PROTOCOL_ID};

pub mod game;
pub mod menu;
pub mod notification;
pub mod splash;

// Enum that will be used as a global state for the game
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    Splash,
    Menu,
    // #[default]
    Game,
}

#[derive(Debug, Component)]
pub struct UiCamera;

/**
 * 初始化默认的ui相机
 */
pub fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), UiCamera));
}

/**
 * 服务器连接配置
 */
#[derive(Debug, Resource, Clone, Eq, PartialEq)]
pub struct ConnectionAddr {
    server: String,
    port: String,
    nickname: String,
}

impl Default for ConnectionAddr {
    fn default() -> Self {
        Self {
            server: String::from("127.0.0.1"),
            port: String::from("5000"),
            nickname: String::from("robzhou"),
        }
    }
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

// 创建连接
pub fn new_renet_client(connection_addr: ConnectionAddr) -> (RenetClient, NetcodeClientTransport) {
    let client = RenetClient::new(connection_config());
    let addr = format!("{}:{}", connection_addr.server, connection_addr.port);
    print!("客户端正在连接:{}", addr);
    let server_addr = addr.parse().unwrap();
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
        user_data: Some(Username(connection_addr.nickname).to_netcode_user_data()),
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    (client, transport)
}

pub const CHINESE: &str = "Chinese";
pub const ENGLISH: &str = "English";
