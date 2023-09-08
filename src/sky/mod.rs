use bevy::{
    prelude::{
        Commands, Component, DirectionalLight, DirectionalLightBundle, IntoSystemConfigs, Plugin,
        Quat, Query, Res, ResMut, Resource, Startup, Transform, Update, Vec3, With,
    },
    time::{Time, Timer, TimerMode},
};
use bevy_atmosphere::{
    prelude::{AtmosphereModel, AtmospherePlugin, Nishita},
    system_param::AtmosphereMut,
};
use bevy_renet::renet::{RenetClient, RenetServer};

use crate::server::message_def::{time_sync::TimeSync, ServerChannel};

#[derive(Component)]
pub struct Sun;

#[derive(Resource)]
pub struct CycleTimer(Timer);

fn daylight_cycle(mut timer: ResMut<CycleTimer>, time: Res<Time>, mut server: ResMut<RenetServer>) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        // todo 这里的更平滑的一天？
        let t = time.elapsed_seconds_wrapped() / 50.0;
        let message = bincode::serialize(&TimeSync::SkyBox(t)).unwrap();
        server.broadcast_message(ServerChannel::TimsSync, message);
    }
}

pub struct ServerSkyPlugins;

impl Plugin for ServerSkyPlugins {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(CycleTimer(Timer::new(
            bevy::utils::Duration::from_millis(50),
            TimerMode::Repeating,
        )));
        app.add_systems(Update, daylight_cycle);
    }
}

// Simple environment
fn setup_environment(mut commands: Commands) {
    // Our Sun
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 100000.0,
                shadows_enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
        Sun, // Marks the light as Sun
    ));
}

pub struct ClientSkyPlugins;

impl Plugin for ClientSkyPlugins {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(AtmosphereModel::new(Nishita::default()));
        app.add_plugins(AtmospherePlugin);
        app.add_systems(Startup, setup_environment);
        app.add_systems(
            Update,
            async_sky.run_if(bevy_renet::transport::client_connected()),
        );
    }
}

fn async_sky(
    mut client: ResMut<RenetClient>,
    mut atmosphere: AtmosphereMut<Nishita>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
) {
    while let Some(message) = client.receive_message(ServerChannel::TimsSync) {
        let time_sync: TimeSync = bincode::deserialize(&message).unwrap();
        match time_sync {
            TimeSync::SkyBox(t) => {
                atmosphere.sun_position = Vec3::new(0., t.sin(), t.cos());
                if let Some((mut light_trans, mut directional)) = query.single_mut().into() {
                    light_trans.rotation = Quat::from_rotation_x(-t);
                    directional.illuminance = t.sin().max(0.0).powf(2.0) * 100000.0;
                }
            }
        }
    }
}
