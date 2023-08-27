use bevy::{
    prelude::{
        in_state, Event, EventReader, EventWriter, IVec3, Input, IntoSystemConfigs, MouseButton,
        Plugin, Res, ResMut, Resource, Update,
    },
    time::{Time, Timer, TimerMode},
};
use bevy_renet::renet::RenetClient;

use crate::{
    client::{
        chunk_query::ChunkQuery, client_channel::ClientChannel, ray_cast::choose_cube::ChooseCube,
        state_manager::GameState, ui::tool_bar::ToolBar,
    },
    tools::vec3_to_chunk_key_any_xyz,
    voxel_world::{chunk::ChunkKey, voxel::Voxel},
};

use super::controller::ControllerFlag;

// 破坏方块的计时器

#[derive(Debug, Resource, Clone)]
pub struct AttackTimer {
    pub pressed: bool,
    pub timer: Option<Timer>,
    pub chunk_key: ChunkKey,
    pub xyz: [u32; 3],
}

#[derive(Debug, Event)]
pub struct BrokeCubeEvent {
    pub chunk_key: ChunkKey,
    pub xyz: [u32; 3],
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
                chunk_key: attack_timer.chunk_key.clone(),
                xyz: attack_timer.xyz.clone(),
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
        })
        .unwrap();
        client.send_message(ClientChannel::ChunkQuery, message);
    }
}

//鼠标操作
pub fn mouse_button_system(
    mouse_button_input: Res<Input<MouseButton>>,
    choose_cube: Res<ChooseCube>,
    controller_flag: Res<ControllerFlag>,
    mut client: ResMut<RenetClient>,
    tool_bar_data: Res<ToolBar>,
    mut attack_timer: ResMut<AttackTimer>,
) {
    if !controller_flag.flag {
        // println!("3:{}", controller_flag.flag);
        return;
    }
    if mouse_button_input.just_pressed(MouseButton::Left) || attack_timer.pressed {
        attack_timer.pressed = true;
        // println!("4:{}", controller_flag.flag);
        // 破坏方块
        if let Some(pos) = choose_cube.center {
            let (chunk_key, xyz) = vec3_to_chunk_key_any_xyz(pos);
            // 判断计时器是否存在
            let test_chunk_key = attack_timer.chunk_key.clone();
            let test_xyz = attack_timer.xyz.clone();
            if let Some(_) = &mut attack_timer.timer {
                // FIXME: 理论上不会走到这个分支
                if test_chunk_key == chunk_key && test_xyz == xyz {
                    // 和原来位置一样不处理
                } else {
                    // FIXME: 后续根据体素块 和 当前物体来判断 新的 timer
                    attack_timer.timer = Some(Timer::new(
                        bevy::utils::Duration::from_millis(1000 * 5),
                        TimerMode::Once,
                    ));
                    attack_timer.chunk_key = chunk_key.clone();
                    attack_timer.xyz = xyz.clone();
                }
            } else {
                attack_timer.chunk_key = chunk_key.clone();
                attack_timer.xyz = xyz.clone();
                // FIXME: 后续根据体素块 和 当前物体来判断
                attack_timer.timer = Some(Timer::new(
                    bevy::utils::Duration::from_millis(1000 * 5),
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
        match tool_bar_data.staff_type() {
            Some(staff_type) => {
                match staff_type {
                    crate::staff::StaffType::Voxel(voxel_type) => {
                        if let Some(pos) = choose_cube.out_center {
                            let (chunk_key, xyz) = vec3_to_chunk_key_any_xyz(pos);
                            let message = bincode::serialize(&ChunkQuery::Change {
                                chunk_key: chunk_key,
                                pos: xyz,
                                voxel_type: voxel_type,
                            })
                            .unwrap();
                            client.send_message(ClientChannel::ChunkQuery, message);
                        }
                    }
                    // TODO: 对其他的物品类型进行处理
                    _ => {}
                }
            }
            None => {}
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
