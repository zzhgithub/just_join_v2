use bevy::{
    prelude::{
        AlphaMode, Assets, Color, Commands, Entity, Gizmos, GlobalTransform, Mesh, PbrBundle,
        Plugin, Query, Res, ResMut, StandardMaterial, Startup, Transform, Update, Vec3, Visibility,
        With, Without,
    },
    reflect::Reflect,
    render::render_resource::PrimitiveTopology,
};
use bevy_mod_raycast::{
    system_param::{Raycast, RaycastSettings},
    DefaultRaycastingPlugin, Ray3d,
};

use crate::{CLIENT_DEBUG, TOUCH_RADIUS};

use self::choose_cube::{ChooseCube, HelpCube};

use super::{
    mesh_display::TerrainMesh,
    player::{controller::CameraTag, mouse_control::AttackTimer},
};

pub mod choose_cube;

fn get_pos_chunk_center(vec3: Vec3, normal: Vec3) -> Vec3 {
    // 应该是命中点所在的面的中点
    let mid_pos = Vec3::new(
        calc_point(vec3.x, normal.x),
        calc_point(vec3.y, normal.y),
        calc_point(vec3.z, normal.z),
    );
    mid_pos - (normal * 0.5)
}

fn calc_point(x: f32, normal: f32) -> f32 {
    // 这里 使用四舍五入进行计算
    ((x * 1000.0).round() / 1000.0).floor() + 0.5 * (if normal == 0.0 { 1.0 } else { 0.0 })
}

#[derive(Reflect)]
pub struct MyRaycastSet;

pub struct MeshRayCastPlugin;

impl Plugin for MeshRayCastPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // 加载资源
        app.add_plugins(DefaultRaycastingPlugin::<MyRaycastSet>::default());
        app.insert_resource(ChooseCube::new());
        app.add_systems(Startup, setup_cube);
        app.add_systems(Update, touth_mesh_ray_cast);
    }
}

pub fn setup_cube(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: mesh_assets.add(create_cube_wireframe(1.001)),
            visibility: bevy::prelude::Visibility::Hidden,
            material: materials.add(StandardMaterial {
                unlit: true,
                base_color: Color::BLACK,
                depth_bias: 9999.0,
                alpha_mode: AlphaMode::Mask(0.5),
                ..Default::default()
            }), // 使用 Wireframe 材质
            transform: Transform::from_translation(Vec3::ZERO),
            ..Default::default()
        })
        .insert(HelpCube);
}

fn create_cube_wireframe(size: f32) -> Mesh {
    let half_size = size / 2.0;

    let vertices = vec![
        [half_size, half_size, half_size],
        [-half_size, half_size, half_size],
        [-half_size, -half_size, half_size],
        [half_size, -half_size, half_size],
        [half_size, half_size, half_size],
        [half_size, half_size, -half_size],
        [-half_size, half_size, -half_size],
        [-half_size, -half_size, -half_size],
        [half_size, -half_size, -half_size],
        [half_size, half_size, -half_size],
        [-half_size, half_size, half_size],
        [-half_size, half_size, -half_size],
        [-half_size, -half_size, half_size],
        [-half_size, -half_size, -half_size],
        [half_size, -half_size, half_size],
        [half_size, -half_size, -half_size],
        [half_size, half_size, half_size],
        [half_size, half_size, -half_size],
    ];

    let indices = vec![
        0, 1, 1, 2, 2, 3, 3, 4, 4, 0, // Front face
        5, 6, 6, 7, 7, 8, 8, 9, 9, 5, // Back face
        10, 11, 12, 13, 14, 15, // Connecting lines
        16, 17,
    ];

    let mut mesh = Mesh::new(PrimitiveTopology::LineList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
    mesh
}

#[allow(clippy::type_complexity)]
pub fn touth_mesh_ray_cast(
    mut raycast: Raycast,
    query: Query<&GlobalTransform, With<CameraTag>>,
    mut choose_cube: ResMut<ChooseCube>,
    mut gizmos: Gizmos,
    query_mesh: Query<Entity, &TerrainMesh>,
    mut query_help_cube: Query<
        (&mut Transform, &mut Visibility),
        (With<HelpCube>, Without<CameraTag>),
    >,
    attack_timer: Res<AttackTimer>,
) {
    let Ok((mut chue_pos, mut visibility)) = query_help_cube.get_single_mut() else {
        println!("not found Cube.");
        return;
    };

    let Ok(tfr) = query.get_single() else {
        return;
    };
    let ray_pos = tfr.translation();
    let ray_dir = tfr.forward();
    let ray = Ray3d::new(ray_pos, ray_dir);

    let hits = raycast.cast_ray(
        ray,
        &RaycastSettings {
            // 遇到第一个就退出
            filter: &|entity| query_mesh.get(entity).is_ok(),
            early_exit_test: &|_| true,
            ..Default::default()
        },
    );

    if let Some((_, hit)) = hits.first() {
        let hit_point = hit.position();
        if ray_pos.distance(hit_point) <= TOUCH_RADIUS {
            let normal = hit.normal();
            if CLIENT_DEBUG {
                gizmos.ray(hit_point, normal, Color::RED);
            }
            gizmos.circle(hit_point, normal, 0.1, Color::BLACK);
            if let Some(timer) = &attack_timer.timer {
                let rate = timer.elapsed().as_millis() as f32 / timer.duration().as_millis() as f32;
                gizmos.circle(hit_point, normal, 0.1 * rate, Color::BLUE);
            }
            let center_point = get_pos_chunk_center(hit_point, normal);
            let out_center_point = get_pos_chunk_center(hit_point, -normal);

            *visibility = Visibility::Visible;
            *chue_pos = Transform::from_translation(center_point);

            choose_cube.choose_on = Some(hit_point);
            choose_cube.center = Some(center_point);
            choose_cube.out_center = Some(out_center_point);
        } else {
            hidden_help_cube(choose_cube.as_mut(), &mut visibility);
        }
    } else {
        hidden_help_cube(choose_cube.as_mut(), &mut visibility);
    }
}

fn hidden_help_cube(choose_cube: &mut ChooseCube, visibility: &mut Visibility) {
    choose_cube.choose_on = None;
    choose_cube.center = None;
    choose_cube.out_center = None;
    *visibility = Visibility::Hidden
}
