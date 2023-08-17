// system that converts delta axis events into pitch and yaw
use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use std::ops::Deref;

#[derive(Debug, Default, Event)]
pub struct PitchEvent {
    pub pitch: f32,
}

impl PitchEvent {
    pub fn new(value: f32) -> Self {
        Self { pitch: value }
    }
}

impl Deref for PitchEvent {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.pitch
    }
}

#[derive(Debug, Default, Event)]
pub struct YawEvent {
    pub yaw: f32,
}

impl YawEvent {
    pub fn new(value: f32) -> Self {
        Self { yaw: value }
    }
}

impl Deref for YawEvent {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.yaw
    }
}

#[derive(Clone, Copy, Component)]
pub struct LookDirection {
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

impl Default for LookDirection {
    fn default() -> Self {
        Self {
            forward: Vec3::Z,
            right: -Vec3::X,
            up: Vec3::Y,
        }
    }
}

#[derive(Debug, Component)]
pub struct LookEntity(pub Entity);

pub fn forward_up(settings: Res<MouseSettings>, mut query: Query<&mut LookDirection>) {
    for mut look in query.iter_mut() {
        let rotation = Quat::from_euler(
            EulerRot::YXZ,
            settings.yaw_pitch_roll.x,
            settings.yaw_pitch_roll.y,
            settings.yaw_pitch_roll.z,
        );
        look.forward = rotation * -Vec3::Z;
        look.right = rotation * Vec3::X;
        look.up = rotation * Vec3::Y;
    }
}
#[derive(Debug, Resource)]
pub struct MouseSettings {
    pub sensitivity: f32,
    pub yaw_pitch_roll: Vec3,
}

impl Default for MouseSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.01,
            yaw_pitch_roll: Vec3::ZERO,
        }
    }
}

const PITCH_BOUND: f32 = std::f32::consts::FRAC_PI_2 - 1E-3;

pub fn input_to_look(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut settings: ResMut<MouseSettings>,
    mut pitch_events: EventWriter<PitchEvent>,
    mut yaw_events: EventWriter<YawEvent>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = primary_window.get_single() {
        match window.cursor.grab_mode {
            CursorGrabMode::None => (),
            _ => {
                let mut delta = Vec2::ZERO;
                for motion in mouse_motion_events.iter() {
                    // NOTE: -= to invert
                    delta -= motion.delta;
                }
                if delta.length_squared() > 1E-6 {
                    delta *= settings.sensitivity;
                    settings.yaw_pitch_roll += delta.extend(0.0);
                    if settings.yaw_pitch_roll.y > PITCH_BOUND {
                        settings.yaw_pitch_roll.y = PITCH_BOUND;
                    }
                    if settings.yaw_pitch_roll.y < -PITCH_BOUND {
                        settings.yaw_pitch_roll.y = -PITCH_BOUND;
                    }
                    
                    pitch_events.send(PitchEvent::new(settings.yaw_pitch_roll.y));
                    yaw_events.send(YawEvent::new(settings.yaw_pitch_roll.x));
                }
            }
        }
    }
}
