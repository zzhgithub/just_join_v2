// 树干

use std::cmp::max;

use bevy::{
    math::{vec2, Vec3Swizzles},
    prelude::{Vec2, Vec3},
};

/**
 * 树干函数
 * root ：根部
 * pos  ：检查点
 * h : 树干占的 体素格子数
 * >0 是外部  <=0 是内部
 */
pub fn trunk(root: Vec3, h: u32) -> impl FnMut(Vec3) -> f32 {
    move |pos: Vec3| {
        let a = pos - root;
        if a.x != 0. || a.z != 0. || a.y < 0. || a.y > h as f32 {
            return 1.0;
        }
        0.
    }
}

/**
 * 获取半圆
 */
pub fn sd_cut_sphere(center: Vec3, r: f32, h: f32) -> impl FnMut(Vec3) -> f32 {
    move |pos: Vec3| {
        let p = pos - center;

        let w = (r * r - h * h).sqrt();

        // sampling dependant computations
        let q = vec2(p.xz().length(), p.y);
        let a = (h - r) * q.x * q.x + w * w * (h + r - 2.0 * q.y);
        let b = h * q.x - w * q.y;
        let s = if a > b { a } else { b };
        return if s < 0.0 {
            q.length() - r
        } else {
            if q.x < w {
                h - q.y
            } else {
                (q - vec2(w, h)).length()
            }
        };
    }
}
