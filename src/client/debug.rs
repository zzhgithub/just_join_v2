use bevy::prelude::{Color, Gizmos, Plugin, Query, Transform, Update, Vec3};

use crate::server::player::Player;

pub struct ClientDebugPlugin;

impl Plugin for ClientDebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, debug_player_aabb);
    }
}

// 查询Player边框
fn debug_player_aabb(mut gizmos: Gizmos, query: Query<(&Player, &Transform)>) {
    for (_, trf) in query.iter() {
        gizmos.cuboid(
            Transform {
                translation: trf.translation.clone(),
                scale: Vec3 {
                    x: 0.3,
                    y: 1.7,
                    z: 0.3,
                },
                ..Default::default()
            },
            Color::YELLOW,
        );
    }
}
