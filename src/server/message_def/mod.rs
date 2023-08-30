// 服务端消息定义
pub mod chunk_result;
pub mod filled_object_message;
pub mod networked_entities;
pub mod server_messages;
pub mod time_sync;

use bevy_renet::renet::{ChannelConfig, SendType};
use std::time::Duration;

/**
 * 服务端 频道
 * 是只有服务端可以发送的信息
 */
pub enum ServerChannel {
    ServerMessages,
    NetworkedEntities,
    ChunkResult,
    TimsSync,
    FilledObjectMessage,
}

impl From<ServerChannel> for u8 {
    fn from(channel_id: ServerChannel) -> Self {
        match channel_id {
            ServerChannel::NetworkedEntities => 0,
            ServerChannel::ServerMessages => 1,
            ServerChannel::ChunkResult => 2,
            ServerChannel::TimsSync => 3,
            ServerChannel::FilledObjectMessage => 4,
        }
    }
}

impl ServerChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                channel_id: Self::NetworkedEntities.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::Unreliable,
            },
            ChannelConfig {
                channel_id: Self::ServerMessages.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(200),
                },
            },
            // FIXME: 这里流量太多了！！
            ChannelConfig {
                channel_id: Self::ChunkResult.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::ReliableUnordered {
                    resend_time: Duration::from_millis(200),
                },
            },
            ChannelConfig {
                channel_id: Self::TimsSync.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::Unreliable,
            },
            ChannelConfig {
                channel_id: Self::FilledObjectMessage.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::Unreliable,
            },
        ]
    }
}
