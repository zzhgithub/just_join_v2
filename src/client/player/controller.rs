use bevy::{
    prelude::{
        in_state, warn, Component, Entity, EventReader, Input, IntoSystemConfigs,
        IntoSystemSetConfigs, KeyCode, Mat4, OnEnter, Plugin, PreUpdate, Query, Res, ResMut,
        Resource, SystemSet, Transform, Update, Vec3, Visibility, With,
    },
    window::{CursorGrabMode, PrimaryWindow, Window},
};
use bevy_egui::EguiSet;
use bevy_renet::renet::RenetClient;

use crate::client::{
    client_channel::ClientChannel, player_input::PlayerInput, state_manager::GameState,
};

use super::{
    look::{
        forward_up, input_to_look, LookDirection, LookEntity, MouseSettings, PitchEvent, YawEvent,
    },
    player_input::InputMap,
};

/**
 * 当前输入影响状态
 */
#[derive(Debug, Default, Clone, Copy)]
pub struct InputState {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub run: bool,
    pub jump: bool,
    pub up: bool,
    pub down: bool,
}

#[derive(Debug, Resource)]
pub struct ControllerFlag {
    pub flag: bool,
}

#[derive(Debug, Component, Clone, Copy)]
pub struct CharacterController {
    pub input_map: InputMap,
    pub fly: bool,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub jump_speed: f32,
    pub velocity: Vec3,
    pub jumping: bool,
    pub input_state: InputState,
}

impl Default for CharacterController {
    fn default() -> Self {
        Self {
            input_map: InputMap::default(),
            fly: false,
            walk_speed: 5.0,
            run_speed: 8.0,
            jump_speed: 5.0,
            velocity: Vec3::ZERO,
            jumping: false,
            input_state: InputState::default(),
        }
    }
}

#[derive(Debug, Component)]
pub struct BodyTag;
// 首摇
#[derive(Debug, Component)]
pub struct YawTag;
// 头部
#[derive(Debug, Component)]
pub struct HeadTag;
// 相机
#[derive(Debug, Component)]
pub struct CameraTag;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ControllerSet {
    InputToEvent,
    InputToLook,
    ForwardUp,
}
pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<PitchEvent>()
            .add_event::<YawEvent>()
            .init_resource::<MouseSettings>()
            .add_systems(OnEnter(GameState::Game), initial_grab_cursor)
            .insert_resource(ControllerFlag { flag: true })
            .configure_sets(
                PreUpdate,
                // chain() will ensure sets run in the order they are listed
                (
                    ControllerSet::InputToEvent,
                    ControllerSet::InputToLook,
                    ControllerSet::ForwardUp,
                )
                    .chain(),
            )
            .add_systems(
                PreUpdate,
                (
                    cursor_grab.after(EguiSet::InitContexts),
                    toggle_third_person,
                    (input_to_send)
                        .in_set(ControllerSet::InputToEvent)
                        .run_if(bevy_renet::transport::client_connected()),
                    (input_to_look).in_set(ControllerSet::InputToLook),
                    (forward_up)
                        .in_set(ControllerSet::ForwardUp)
                        .after(ControllerSet::InputToEvent)
                        .after(ControllerSet::InputToLook),
                )
                    .run_if(in_state(GameState::Game)),
            );
        // 发送message系统
        app.add_systems(
            Update,
            (controller_to_yaw, controller_to_pitch)
                .run_if(bevy_renet::transport::client_connected()),
        );
    }
}

// 初始化光标
fn initial_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        toggle_grab_cursor(&mut window);
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    match window.cursor.grab_mode {
        CursorGrabMode::None => {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;
        }
        _ => {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
}

// 光标显示或者隐藏系统
fn cursor_grab(
    mut flags: ResMut<ControllerFlag>,
    keys: Res<Input<KeyCode>>,
    controller_query: Query<&CharacterController>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    for controller in controller_query.iter() {
        let input_map = controller.input_map;
        if let Ok(mut window) = primary_window.get_single_mut() {
            if keys.just_pressed(input_map.toggle_grab_cursor) {
                toggle_grab_cursor(&mut window);
                // println!("1:{}", flags.flag);
                // 添加其他按钮是否生效逻辑
                flags.as_mut().flag = !flags.flag;
                // println!("2:{}", flags.flag);
            }
        } else {
            warn!("Primary window not found for `cursor_grab`!");
        }
    }
}

pub fn controller_to_yaw(mut yaws: EventReader<YawEvent>, mut client: ResMut<RenetClient>) {
    if let Some(yaw) = yaws.iter().next() {
        // transform.rotation = Quat::from_rotation_y(**yaw);
        let message = bincode::serialize(&PlayerInput::YAW(yaw.yaw)).unwrap();
        client.send_message(ClientChannel::Input, message);
    }
}

pub fn controller_to_pitch(mut pitches: EventReader<PitchEvent>, mut client: ResMut<RenetClient>) {
    if let Some(pitch) = pitches.iter().next() {
        // transform.rotation = Quat::from_rotation_x(**pitch);
        let message = bincode::serialize(&PlayerInput::PITCH(pitch.pitch)).unwrap();
        client.send_message(ClientChannel::Input, message);
    }
}

#[derive(Debug, Component)]
pub struct ThirdPerson {
    pub is_third_person: bool,
    pub body: Entity,
    pub head: Entity,
}

fn toggle_third_person(
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_transforms: Query<(&mut Transform, &mut ThirdPerson)>,
    controller_flag: Res<ControllerFlag>,
    mut models: Query<&mut Visibility>,
) {
    // 如果是不能控制状态禁止控制
    if !controller_flag.flag {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::T) {
        for (mut camera_transform, mut third_person) in camera_transforms.iter_mut() {
            third_person.is_third_person = !third_person.is_third_person;
            *camera_transform = Transform::from_matrix(if third_person.is_third_person {
                if let Ok(mut visible) = models.get_mut(third_person.body) {
                    *visible = Visibility::Inherited;
                }
                if let Ok(mut visible) = models.get_mut(third_person.head) {
                    *visible = Visibility::Inherited;
                }
                let eye = -Vec3::Z * 2.0;
                let center = -Vec3::Z * 10.0;
                Mat4::look_to_rh(eye, center, Vec3::Y)
            } else {
                if let Ok(mut visible) = models.get_mut(third_person.body) {
                    *visible = Visibility::Hidden;
                }
                if let Ok(mut visible) = models.get_mut(third_person.head) {
                    *visible = Visibility::Hidden;
                }
                Mat4::look_to_rh(Vec3::ZERO, -Vec3::Z, Vec3::Y)
            });
        }
    }
}

pub fn input_to_send(
    keyboard_input: Res<Input<KeyCode>>,
    mut controller_query: Query<(&LookEntity, &mut CharacterController)>,
    look_direction_query: Query<&LookDirection>,
    controller_flag: Res<ControllerFlag>,
    mut client: ResMut<RenetClient>,
) {
    let xz = Vec3::new(1.0, 0.0, 1.0);
    if !controller_flag.flag {
        return;
    }
    for (look_entity, mut controller) in controller_query.iter_mut() {
        if keyboard_input.just_pressed(controller.input_map.key_fly) {
            controller.fly = !controller.fly;
        }
        if keyboard_input.pressed(controller.input_map.key_forward) {
            controller.input_state.forward = true;
        }
        if keyboard_input.pressed(controller.input_map.key_backward) {
            controller.input_state.backward = true;
        }
        if keyboard_input.pressed(controller.input_map.key_right) {
            controller.input_state.right = true;
        }
        if keyboard_input.pressed(controller.input_map.key_left) {
            controller.input_state.left = true;
        }
        if keyboard_input.pressed(controller.input_map.key_run) {
            controller.input_state.run = true;
        }
        if keyboard_input.just_pressed(controller.input_map.key_jump) {
            controller.input_state.jump = true;
        }
        if keyboard_input.pressed(controller.input_map.key_fly_up) {
            controller.input_state.up = true;
        }
        if keyboard_input.pressed(controller.input_map.key_fly_down) {
            controller.input_state.down = true;
        }

        let look = look_direction_query
            .get_component::<LookDirection>(look_entity.0)
            .expect("Failed to get LookDirection from Entity");

        // Calculate forward / right / up vectors
        let (forward, right, up) = if controller.fly {
            (look.forward, look.right, look.up)
        } else {
            (
                (look.forward * xz).normalize(),
                (look.right * xz).normalize(),
                Vec3::Y,
            )
        };

        // Calculate the desired velocity based on input
        let mut desired_velocity = Vec3::ZERO;
        if controller.input_state.forward {
            desired_velocity += forward;
        }
        if controller.input_state.backward {
            desired_velocity -= forward;
        }
        if controller.input_state.right {
            desired_velocity += right;
        }
        if controller.input_state.left {
            desired_velocity -= right;
        }
        if controller.input_state.up {
            desired_velocity += up;
        }
        if controller.input_state.down {
            desired_velocity -= up;
        }

        // Limit x/z velocity to walk/run speed
        let speed = if controller.input_state.run {
            controller.run_speed
        } else {
            controller.walk_speed
        };
        desired_velocity = if desired_velocity.length_squared() > 1E-6 {
            desired_velocity.normalize() * speed
        } else {
            // No input - apply damping to the x/z of the current velocity
            controller.velocity * 0.5 * xz
        };

        if !controller.fly {
            desired_velocity.y = if controller.input_state.jump {
                controller.jumping = true;
                controller.jump_speed
            } else {
                0.0
            };
        }
        //TODO Handle jumping
        // let was_jumping = controller.jumping;
        let message = bincode::serialize(&PlayerInput::MOVE(desired_velocity)).unwrap();
        client.send_message(ClientChannel::Input, message);
        controller.input_state = InputState::default();
    }
}

// todo 个人的fly模式
