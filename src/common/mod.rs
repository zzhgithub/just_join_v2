use std::marker::PhantomData;

use bevy::{
    prelude::{Component, Plugin, PreUpdate, Query, ResMut, Resource, Transform, Vec3, With},
    reflect::Reflect,
    utils::HashMap,
};
use bevy_inspector_egui::InspectorOptions;

use crate::server::player::Player;

use self::config::VIEW_RADIUS;

pub mod config;

#[derive(Debug, Clone, Copy, Reflect, InspectorOptions)]
pub struct Sphere3 {
    pub center: Vec3,
    pub radius: f32,
}

#[derive(Debug, Resource, Clone, Copy, Reflect, InspectorOptions)]
pub struct ClipSpheres {
    pub old_sphere: Sphere3,
    pub new_sphere: Sphere3,
}

// 适用于单个数据
pub fn update_clip_shpere_system<T>(
    mut clip_spheres: ResMut<ClipSpheres>,
    query: Query<&Transform, With<T>>,
) where
    T: Component,
{
    let position = if let Some(trf) = query.iter().next() {
        trf.translation
    } else {
        return;
    };
    clip_spheres.old_sphere = clip_spheres.new_sphere;
    clip_spheres.new_sphere = Sphere3 {
        center: position,
        radius: VIEW_RADIUS,
    }
}

pub struct ClientClipSpheresPlugin<T> {
    pub data: PhantomData<T>,
}

// 客户端绑定一个数据的 插件
impl<T: Component> Plugin for ClientClipSpheresPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        // init resource of clip Spheres
        let eye = Vec3::ZERO;
        let init_shpere = Sphere3 {
            center: eye,
            radius: VIEW_RADIUS,
        };

        let clip_spheres = ClipSpheres {
            old_sphere: init_shpere,
            new_sphere: init_shpere,
        };
        app.insert_resource(clip_spheres);
        app.add_systems(PreUpdate, update_clip_shpere_system::<T>);
    }
}

#[derive(Debug, Clone, Resource, Reflect)]
pub struct ServerClipSpheres {
    pub clip_spheres: HashMap<u64, ClipSpheres>,
}

pub struct ServerClipSpheresPlugin;

impl Plugin for ServerClipSpheresPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ServerClipSpheres {
            clip_spheres: HashMap::default(),
        });
        // 添加一下角色位置的相关接口
        app.add_systems(PreUpdate, update_all_clip_shpere_system);
    }
}

pub fn update_all_clip_shpere_system(
    mut server_clip_spheres: ResMut<ServerClipSpheres>,
    query: Query<(&Player, &Transform)>,
) {
    for (player, transform) in query.iter() {
        let client_id = player.id;
        let sphere = Sphere3 {
            center: transform.translation,
            radius: VIEW_RADIUS,
        };
        if let Some(clip_sphere) = server_clip_spheres.clip_spheres.get_mut(&client_id) {
            clip_sphere.old_sphere = clip_sphere.new_sphere;
            clip_sphere.new_sphere = sphere;
        } else {
            server_clip_spheres.clip_spheres.insert(
                client_id,
                ClipSpheres {
                    old_sphere: sphere,
                    new_sphere: sphere,
                },
            );
        }
    }
}
