use bevy::{
    prelude::{
        in_state, warn, Event, EventReader, EventWriter, IVec3, Input, IntoSystemConfigs, KeyCode,
        MouseButton, Plugin, Query, Res, ResMut, Resource, Transform, Update, Vec3,
    },
    time::{Time, Timer, TimerMode},
};
use bevy_renet::renet::RenetClient;

use crate::{
    client::{
        message_def::{chunk_query::ChunkQuery, ClientChannel},
        ray_cast::choose_cube::ChooseCube,
        state_manager::GameState,
        ui::tool_bar::ToolBar,
    },
    server::player::Player,
    tools::{vec3_to_chunk_key_any_xyz, zone::check_player_put_object_available},
    voxel_world::{chunk::ChunkKey, chunk_map::ChunkMap, voxel::Voxel},
};

use super::controller::ControllerFlag;

// 破坏方块的计时器

#[derive(Debug, Resource, Clone)]
pub struct AttackTimer {
    pub pressed: bool,
    pub timer: Option<Timer>,
    pub chunk_key: ChunkKey,
    pub xyz: [u32; 3],
    pub center: Vec3,
}

#[derive(Debug, Event)]
pub struct BrokeCubeEvent {
    pub chunk_key: ChunkKey,
    pub xyz: [u32; 3],
    pub center: Vec3,
}

// 处理时间相关
pub fn deal_attack_time(
    time: Res<Time>,
    mut attack_timer: ResMut<AttackTimer>,
    mut broke_cube_event: EventWriter<BrokeCubeEvent>,
) {
    if let Some(timer) = &mut attack_timer.timer {
        timer.tick(time.delta());
        if timer.finished() {
            // 这处理完毕了 要删除物体了！
            broke_cube_event.send(BrokeCubeEvent {
                chunk_key: attack_timer.chunk_key,
                xyz: attack_timer.xyz,
                center: attack_timer.center,
            });
            attack_timer.timer = None;
        }
    }
}

pub fn deal_broken_cube_event(
    mut broke_cube_event: EventReader<BrokeCubeEvent>,
    mut client: ResMut<RenetClient>,
) {
    for event in broke_cube_event.iter() {
        let message = bincode::serialize(&ChunkQuery::Change {
            chunk_key: event.chunk_key,
            pos: event.xyz,
            voxel_type: Voxel::EMPTY,
            center: event.center,
            active_index: None,
        })
        .unwrap();
        client.send_message(ClientChannel::ChunkQuery, message);
    }
}

//鼠标操作
pub fn mouse_button_system(
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    choose_cube: Res<ChooseCube>,
    controller_flag: Res<ControllerFlag>,
    mut client: ResMut<RenetClient>,
    tool_bar_data: Res<ToolBar>,
    mut attack_timer: ResMut<AttackTimer>,
    player_query: Query<(&Player, &Transform)>,
    chunk_map: Res<ChunkMap>,
) {
    if !controller_flag.flag {
        // println!("3:{}", controller_flag.flag);
        return;
    }

    // 移动数据的方向
    if mouse_button_input.just_released(MouseButton::Left)
        && keyboard_input.pressed(KeyCode::ShiftLeft)
    {
        if let Some(pos) = choose_cube.center {
            let (chunk_key, xyz) = vec3_to_chunk_key_any_xyz(pos);
            if let Some(voxel_type) = chunk_map.get_block(chunk_key, xyz) {
                // FIXME: 这里要判断是否每种情况都可以去旋转!
                let new_voxel = voxel_type.next_direction();
                println!("这里发送了旋转方块的指令{:?}", new_voxel);
                let message = bincode::serialize(&ChunkQuery::Change {
                    chunk_key,
                    pos: xyz,
                    voxel_type: new_voxel,
                    center: pos,
                    active_index: None,
                })
                .unwrap();
                client.send_message(ClientChannel::ChunkQuery, message);
            }
        }
    }

    if mouse_button_input.just_pressed(MouseButton::Left) || attack_timer.pressed {
        attack_timer.pressed = true;
        // println!("4:{}", controller_flag.flag);
        // 破坏方块
        if let Some(pos) = choose_cube.center {
            let (chunk_key, xyz) = vec3_to_chunk_key_any_xyz(pos);
            // 判断计时器是否存在
            let test_chunk_key = attack_timer.chunk_key;
            let test_xyz = attack_timer.xyz;
            if attack_timer.timer.is_some() {
                // FIXME: 理论上不会走到这个分支
                if test_chunk_key == chunk_key && test_xyz == xyz {
                    // 和原来位置一样不处理
                } else {
                    // FIXME: 后续根据体素块 和 当前物体来判断 新的 timer
                    attack_timer.timer = Some(Timer::new(
                        bevy::utils::Duration::from_millis(1000 * 2),
                        TimerMode::Once,
                    ));
                    attack_timer.chunk_key = chunk_key;
                    attack_timer.xyz = xyz;
                    attack_timer.center = pos;
                }
            } else {
                attack_timer.chunk_key = chunk_key;
                attack_timer.xyz = xyz;
                attack_timer.center = pos;
                // FIXME: 后续根据体素块 和 当前物体来判断
                attack_timer.timer = Some(Timer::new(
                    bevy::utils::Duration::from_millis(1000 * 2),
                    TimerMode::Once,
                ));
            }
        } else {
            // 清空计时器
            attack_timer.timer = None;
        }
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        // 置空计时器
        attack_timer.pressed = false;
        attack_timer.timer = None;
    }

    if mouse_button_input.just_pressed(MouseButton::Right) {
        // Note: 这里放置时尝试转换成体素再传递
        if let Some(crate::staff::StaffType::Voxel(voxel_type)) =
            tool_bar_data.staff_type_try_to_voxel()
        {
            if let Some(pos) = choose_cube.out_center {
                println!("放置物品{:?}", voxel_type);
                // 判断当前这里是否和 其他的player的位置冲突
                if check_player_put_object_available(pos.clone(), &player_query) {
                    // 还要发送当前生效的 tool_bar的 index
                    let (chunk_key, xyz) = vec3_to_chunk_key_any_xyz(pos);
                    let message = bincode::serialize(&ChunkQuery::Change {
                        chunk_key,
                        pos: xyz,
                        voxel_type,
                        center: pos,
                        active_index: Some(tool_bar_data.active_index),
                    })
                    .unwrap();
                    client.send_message(ClientChannel::ChunkQuery, message);
                } else {
                    warn!("放置物体时有其他的玩家");
                }
            }
        }

        // 这里按下了鼠标右边键 创建方块
    }
}

pub struct MouseControlPlugin;

impl Plugin for MouseControlPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<BrokeCubeEvent>();
        app.insert_resource(AttackTimer {
            pressed: false,
            timer: None,
            chunk_key: ChunkKey(IVec3::ONE),
            xyz: [0, 0, 0],
            center: Vec3::ZERO,
        });
        app.add_systems(
            Update,
            (
                mouse_button_system,
                deal_attack_time,
                deal_broken_cube_event,
            )
                .run_if(bevy_renet::transport::client_connected())
                .run_if(in_state(GameState::Game)),
        );
    }
}
