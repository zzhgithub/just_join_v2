use bevy_renet::renet::{transport::NETCODE_KEY_BYTES, ConnectionConfig};
use client::client_channel::ClientChannel;
use server::server_channel::ServerChannel;

pub mod client;
pub mod common;
pub mod server;
pub mod tools;
pub mod voxel_world;

pub const PRIVATE_KEY: &[u8; NETCODE_KEY_BYTES] = b"an example very very secret key."; // 32-bytes
pub const WORD_PATH: &str = "world_test";
pub const MATERIAL_RON: &str = "volex.ron";
pub const PROTOCOL_ID: u64 = 7;

pub type SmallKeyHashMap<K, V> = ahash::AHashMap<K, V>;

// 可视半径
pub const VIEW_RADIUS: f32 = 128.00;
// 物理引擎半径 这里如果计算的慢可能 跟不上？
pub const PY_DISTANCE: i32 = 1;
// CHUNK大小
pub const CHUNK_SIZE: i32 = 16;
pub const CHUNK_SIZE_U32: u32 = CHUNK_SIZE as u32;
pub const CHUNK_SIZE_ADD_2_U32: u32 = CHUNK_SIZE_U32 + 2;
// 贴图个数
pub const MAX_TEXTURE_COUNT: usize = 9;

pub fn connection_config() -> ConnectionConfig {
    ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        client_channels_config: ClientChannel::channels_config(),
        server_channels_config: ServerChannel::channels_config(),
    }
}
