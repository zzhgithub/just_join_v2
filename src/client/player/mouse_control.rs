use bevy::prelude::{Input, MouseButton, Res, ResMut};
use bevy_renet::renet::RenetClient;

use crate::{
    client::{
        chunk_query::ChunkQuery, client_channel::ClientChannel, ray_cast::choose_cube::ChooseCube,
    },
    tools::vec3_to_chunk_key_any_xyz,
    voxel_world::voxel::{Stone, Voxel, VoxelMaterial},
};

use super::controller::ControllerFlag;

//鼠标操作
pub fn mouse_button_system(
    mouse_button_input: Res<Input<MouseButton>>,
    choose_cube: Res<ChooseCube>,
    controller_flag: Res<ControllerFlag>,
    mut client: ResMut<RenetClient>,
) {
    if !controller_flag.flag {
        return;
    }
    if mouse_button_input.just_pressed(MouseButton::Left) {
        // 破坏方块
        if let Some(pos) = choose_cube.center {
            let (chunk_key, xyz) = vec3_to_chunk_key_any_xyz(pos);
            let message = bincode::serialize(&ChunkQuery::Change {
                chunk_key: chunk_key,
                pos: xyz,
                voxel_type: Voxel::EMPTY,
            })
            .unwrap();
            client.send_message(ClientChannel::ChunkQuery, message);
        }
    }
    if mouse_button_input.just_pressed(MouseButton::Right) {
        // 这里按下了鼠标右边键 创建方块
        if let Some(pos) = choose_cube.out_center {
            let (chunk_key, xyz) = vec3_to_chunk_key_any_xyz(pos);
            let message = bincode::serialize(&ChunkQuery::Change {
                chunk_key: chunk_key,
                pos: xyz,
                //FIXME: 这里后续要改成 通过当前物品栏来获取
                voxel_type: Stone::into_voxel(),
            })
            .unwrap();
            client.send_message(ClientChannel::ChunkQuery, message);
        }
    }
}
