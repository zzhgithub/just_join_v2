// 区域检查

use bevy::prelude::{Query, Transform, Vec3};

use crate::server::player::Player;

pub struct Zone {
    pub center: Vec3,
    pub width_x: f32,
    pub width_z: f32,
    pub height: f32,
    pub min: Vec3,
    pub max: Vec3,
}

impl Zone {
    pub fn new(center: Vec3, width_x: f32, width_z: f32, height: f32) -> Self {
        Self {
            center,
            width_x,
            width_z,
            height,
            min: Vec3 {
                x: center.x - width_x / 2.0,
                y: center.y - height / 2.0,
                z: center.z - width_z / 2.0,
            },
            max: Vec3 {
                x: center.x + width_x / 2.0,
                y: center.y + height / 2.0,
                z: center.z + width_z / 2.0,
            },
        }
    }

    pub fn cuboid(center: Vec3, scale: Vec3) -> Self {
        Self::new(center, scale.x, scale.z, scale.y)
    }

    // 是否相交
    pub fn intersect(&self, other: &Zone) -> bool {
        if self.max.x < other.min.x || self.min.x > other.max.x {
            return false;
        }
        if self.max.y < other.min.y || self.min.y > other.max.y {
            return false;
        }
        if self.max.z < other.min.z || self.min.z > other.max.z {
            return false;
        }
        true
    }
}

// 检查用户是否可以放置物品
pub fn check_player_put_object_available(
    pos: Vec3,
    player_query: &Query<(&Player, &Transform)>,
) -> bool {
    let put_zone = Zone::cuboid(pos, Vec3::ONE);
    for (_, trf) in player_query.iter() {
        let check_zone = Zone::cuboid(
            trf.translation,
            Vec3 {
                x: 0.6,
                y: 1.7,
                z: 0.6,
            },
        );
        if put_zone.intersect(&check_zone) {
            return false;
        }
    }
    true
}
