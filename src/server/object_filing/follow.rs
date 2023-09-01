// 设置物品跟随相关

// 定义状态
use bevy::{
    prelude::{
        Commands, Component, Entity, Plugin, PreUpdate, Query, Res, ResMut, Transform, Update,
        With, Without,
    },
    reflect::Reflect,
    time::Time,
};
use bevy_rapier3d::{
    prelude::{RapierContext, RapierRigidBodyHandle},
    rapier::prelude::RigidBodyType,
};
use bevy_renet::renet::RenetServer;
use seldom_state::{
    prelude::StateMachine,
    trigger::{BoolTrigger, OptionTrigger, Trigger},
};

use crate::{
    server::{
        message_def::{tool_bar_message::ToolBarMessage, ServerChannel},
        player::Player,
    },
    voxel_world::player_state::PlayerOntimeState,
    CLOSE_RANGE, NEAR_RANGE, PICK_SPEED,
};

use super::FilledObject;

#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
struct Idle;

#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
struct Follow {
    target: Entity,
    speed: f32,
}

#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
struct Picked {
    target: Entity,
}

// 目标 距离
#[derive(Clone, Copy)]
struct Near {
    range: f32,
}

// 目标 接近
#[derive(Clone, Copy)]
struct CloseTo {
    range: f32,
}

impl OptionTrigger for Near {
    type Param<'w, 's> = (
        Query<'w, 's, (Entity, &'static Transform, &'static Player), Without<FilledObject>>,
        Query<'w, 's, &'static Transform, With<FilledObject>>,
    );
    type Some = Entity;

    fn trigger(
        &self,
        entity: Entity,
        (player_query, mut filled_query): Self::Param<'_, '_>,
    ) -> Option<Self::Some> {
        let mut min_pair: Option<(Entity, f32)> = None;
        if let Ok(from) = filled_query.get_mut(entity) {
            for (player_entity, to, _) in player_query.iter() {
                let dis = from.translation.distance(to.translation);
                if let Some((_, old_distance)) = min_pair.clone() {
                    if dis < old_distance {
                        min_pair = Some((player_entity, dis));
                    }
                } else {
                    min_pair = Some((player_entity, dis));
                }
            }
        }
        if let Some((entity, min)) = min_pair {
            if min <= self.range {
                return Some(entity);
            }
        }
        None
    }
}

impl BoolTrigger for CloseTo {
    type Param<'w, 's> = (
        Query<'w, 's, &'static Transform>,
        Query<'w, 's, &'static Follow>,
    );

    fn trigger(&self, entity: Entity, (query, follow_query): Self::Param<'_, '_>) -> bool {
        if let Ok(form) = query.get(entity) {
            if let Ok(follow) = follow_query.get(entity) {
                if let Ok(to) = query.get(follow.target) {
                    let distance = form.translation.distance(to.translation);
                    if distance < self.range {
                        return true;
                    }
                }
            }
        }
        false
    }
}

pub struct ObjectFilingFollowPlugin;

impl Plugin for ObjectFilingFollowPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            PreUpdate,
            (
                load_up_state_machine,
                follow_change_type,
                not_follow_change_type,
            ),
        );
        app.add_systems(Update, (follow_entity, pick_up_entity));
    }
}

fn load_up_state_machine(
    mut commands: Commands,
    query: Query<(Entity, &FilledObject), Without<StateMachine>>,
) {
    for (entity, _) in query.iter() {
        commands.entity(entity).insert(Idle).insert(
            StateMachine::default()
                .trans_builder(Near { range: NEAR_RANGE }, |_: &Idle, entity: Entity| {
                    Some(Follow {
                        target: entity,
                        speed: PICK_SPEED,
                    })
                })
                .trans::<Follow>(Near { range: NEAR_RANGE }.not(), Idle)
                .trans_builder(CloseTo { range: CLOSE_RANGE }, |follow: &Follow, _| {
                    Some(Picked {
                        target: follow.target,
                    })
                }),
        );
    }
}

fn follow_entity(
    mut transforms: Query<&mut Transform>,
    follows: Query<(Entity, &Follow)>,
    time: Res<Time>,
) {
    for (entity, follow) in &follows {
        // Get the positions of the follower and target
        let target_translation = transforms.get(follow.target).unwrap().translation;
        let follow_transform = &mut transforms.get_mut(entity).unwrap();
        let follow_translation = follow_transform.translation;

        follow_transform.translation += (target_translation - follow_translation)
            .normalize_or_zero()
            * follow.speed
            * time.delta_seconds();
    }
}

fn follow_change_type(
    mut context: ResMut<RapierContext>,
    query: Query<(Entity, &RapierRigidBodyHandle, &FilledObject, &Follow)>,
) {
    for (_, handle, _, _) in query.iter() {
        if let Some(body) = context.bodies.get_mut(handle.0) {
            match body.body_type() {
                bevy_rapier3d::rapier::prelude::RigidBodyType::KinematicPositionBased => {}
                _ => {
                    body.set_body_type(RigidBodyType::KinematicPositionBased, true);
                }
            }
        }
    }
}

fn not_follow_change_type(
    mut context: ResMut<RapierContext>,
    query: Query<(Entity, &RapierRigidBodyHandle, &FilledObject, &Idle)>,
) {
    for (_, handle, _, _) in query.iter() {
        if let Some(body) = context.bodies.get_mut(handle.0) {
            match body.body_type() {
                bevy_rapier3d::rapier::prelude::RigidBodyType::Dynamic => {}
                _ => {
                    body.set_body_type(RigidBodyType::Dynamic, true);
                }
            }
        }
    }
}

fn pick_up_entity(
    mut commands: Commands,
    // 有状态的角色
    mut palyer_states: Query<(Entity, &Player, &mut PlayerOntimeState)>,
    // 被捡起的数据
    pick_query: Query<(Entity, &FilledObject, &Picked)>,
    mut server: ResMut<RenetServer>,
) {
    for (pick_entity, filled_object, picked) in pick_query.iter() {
        // 1. 获取到pick的目标受体
        if let Ok((_, player, mut player_state)) = palyer_states.get_mut(picked.target) {
            // 2. 检查可以使用的空位 并修改数据
            if let Some((index, _, num)) = player_state.0.put_statff(filled_object.staff.id) {
                // 找到位置并摆放
                // 发送消息销毁对象
                let message = bincode::serialize(&ToolBarMessage::SyncToolbar {
                    index: index,
                    staff_id: Some(filled_object.staff.id),
                    num: num,
                })
                .unwrap();
                server.send_message(player.id, ServerChannel::ToolBarMessage, message);
                commands.entity(pick_entity).despawn();
            } else {
                // 没有找到位置 重新回到idle状态
                commands.entity(pick_entity).remove::<Picked>().insert(Idle);
            }
        }
    }
}
