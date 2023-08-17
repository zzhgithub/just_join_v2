use bevy::input::keyboard::KeyCode;

#[derive(Debug, Clone, Copy)]
pub struct InputMap {
    pub key_forward: KeyCode,
    pub key_backward: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_jump: KeyCode,
    pub key_run: KeyCode,
    // 蹲
    pub key_crouch: KeyCode,
    // invert 倒转
    pub invert_y: bool,
    pub key_fly: KeyCode,
    pub key_fly_up: KeyCode,
    pub key_fly_down: KeyCode,
    pub toggle_grab_cursor: KeyCode,
}

impl Default for InputMap {
    fn default() -> Self {
        Self {
            key_forward: KeyCode::W,
            key_backward: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_jump: KeyCode::Space,
            // 跑
            key_run: KeyCode::ShiftLeft,
            // 蹲
            key_crouch: KeyCode::ControlLeft,
            invert_y: false,
            key_fly: KeyCode::F,
            key_fly_up: KeyCode::E,
            key_fly_down: KeyCode::Q,
            toggle_grab_cursor: KeyCode::Escape,
        }
    }
}
