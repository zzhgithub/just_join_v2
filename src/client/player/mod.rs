use bevy::{
    prelude::{
        shape, Assets, BuildChildren, Camera3dBundle, Color, Commands, ComputedVisibility, Entity,
        GlobalTransform, Mat4, Mesh, PbrBundle, Quat, Resource, StandardMaterial, Transform, Vec3,
        Visibility,
    },
    text::{Text, TextAlignment, TextSection, TextStyle},
    transform::TransformBundle,
    utils::HashMap,
};
use bevy_atmosphere::prelude::AtmosphereCamera;
use bevy_mod_billboard::BillboardTextBundle;

use crate::server::player::Player;

use self::{
    controller::{BodyTag, CameraTag, CharacterController, HeadTag, ThirdPerson, YawTag},
    look::{LookDirection, LookEntity},
};

pub mod controller;
pub mod look;
pub mod mouse_control;
pub mod player_input;
pub mod throw_system;

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    // 客户端 实体
    pub client_entity: Entity,
    // 服务端 实体
    pub server_entity: Entity,
}

// 客户端 玩家大厅！
#[derive(Debug, Default, Resource)]
pub struct ClientLobby {
    pub players: HashMap<u64, PlayerInfo>,
    pub yaws: HashMap<u64, Entity>,
    pub pitch: HashMap<u64, Entity>,
}

pub fn client_create_player(
    commands: &mut Commands,
    transform: Transform,
    client_id: u64,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
    username: String,
    is_current: bool,
) -> (Entity, Entity, Entity) {
    let box_y = 1.0;
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let red = materials.add(Color::hex("800000").unwrap().into());

    let mut body_entry = commands.spawn(Player {
        id: client_id,
        username: username.clone(),
    });
    body_entry
        .insert(BodyTag)
        .insert(TransformBundle::from(transform))
        .insert((Visibility::Inherited, ComputedVisibility::HIDDEN));
    if is_current {
        body_entry.insert(CharacterController::default());
    }
    let body = body_entry.id();
    let yaw = commands
        .spawn((
            GlobalTransform::IDENTITY,
            Transform::IDENTITY,
            YawTag,
            Visibility::Inherited,
            ComputedVisibility::HIDDEN,
        ))
        .id();
    let body_model = commands
        .spawn(PbrBundle {
            material: red.clone(),
            mesh: cube.clone(),
            transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::new(0.5, 1.9, 0.3) - 0.3 * Vec3::Y,
                Quat::IDENTITY,
                Vec3::new(0.0, 0.5 * (box_y + 1.9 - 0.3) - 1.695, 0.0),
            )),
            visibility: Visibility::Inherited,
            ..Default::default()
        })
        .id();
    let head = commands
        .spawn((
            GlobalTransform::IDENTITY,
            Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::from_rotation_y(0.0),
                Vec3::new(0.0, 0.5 * (box_y - 0.3) + 1.9 - 1.695, 0.0),
            )),
            HeadTag,
            Visibility::Inherited,
            ComputedVisibility::HIDDEN,
        ))
        .id();
    let head_model = commands
        .spawn(PbrBundle {
            material: red,
            mesh: cube,
            transform: Transform::from_scale(Vec3::splat(0.3)),
            visibility: Visibility::Inherited,
            ..Default::default()
        })
        .id();

    let color = if is_current {
        Color::GREEN
    } else {
        Color::YELLOW
    };
    // 这里要添加广告牌！
    let billboard = commands
        .spawn(BillboardTextBundle {
            text: Text::from_sections([TextSection {
                value: format!("[{}]", username),
                style: TextStyle {
                    font_size: 30.0,
                    color,
                    ..Default::default()
                },
            }])
            .with_alignment(TextAlignment::Center),
            transform: Transform {
                translation: Vec3::new(0., 0.8, 0.),
                scale: Vec3::splat(0.0035),
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
    if is_current {
        let eye = -Vec3::Z * 2.0;
        let center = -Vec3::Z * 10.0;
        let camera = commands
            .spawn(Camera3dBundle {
                transform: Transform::from_matrix(Mat4::look_to_rh(eye, center, Vec3::Y)),
                ..Default::default()
            })
            .insert(ThirdPerson {
                is_third_person: true,
                body,
                head,
            })
            // 天空合作还没有启用
            .insert(AtmosphereCamera::default())
            .insert((LookDirection::default(), CameraTag))
            .id();
        commands
            .entity(body)
            .insert(LookEntity(camera))
            .push_children(&[yaw, billboard]);
        commands.entity(yaw).push_children(&[body_model, head]);
        commands.entity(head).push_children(&[head_model, camera]);
    } else {
        commands.entity(body).push_children(&[yaw, billboard]);
        commands.entity(yaw).push_children(&[body_model, head]);
        commands.entity(head).push_children(&[head_model]);
    }
    (body, yaw, head)
}
