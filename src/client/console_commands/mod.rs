use bevy::prelude::{App, EventReader, IntoSystemConfigs, Plugin, PreUpdate, Res, ResMut, Update};
use bevy_console::{
    AddConsoleCommand, ConsoleCommandEntered, ConsoleOpen, ConsolePlugin, ConsoleSet,
};

use self::mesh_state::{check_mesh_state, MeshStateCommand};

use super::player::controller::ControllerFlag;

pub mod mesh_state;

pub struct ConsoleCommandPlugins;

impl Plugin for ConsoleCommandPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConsolePlugin)
            .add_systems(PreUpdate, sync_flags)
            .add_systems(Update, raw_commands.in_set(ConsoleSet::Commands))
            .add_console_command::<MeshStateCommand, _>(check_mesh_state);
    }
}

// 保持打开时不能操作人物
fn sync_flags(mut controller_flag: ResMut<ControllerFlag>, console_open: Res<ConsoleOpen>) {
    if console_open.open {
        controller_flag.flag = false;
    } else {
        // 这里怎么知道其他的状态呢？
    }
}

fn raw_commands(mut console_commands: EventReader<ConsoleCommandEntered>) {
    for ConsoleCommandEntered { command_name, args } in console_commands.iter() {
        println!(r#"Entered command "{command_name}" with args {:#?}"#, args);
    }
}
