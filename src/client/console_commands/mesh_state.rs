use bevy::prelude::Res;
use bevy_console::ConsoleCommand;
use clap::Parser;

use crate::{
    client::mesh_display::MeshManager,
    common::ClipSpheres,
    voxel_world::{
        chunk::{get_chunk_key_i3_by_vec3, ChunkKey},
        chunk_map::ChunkMap,
    },
};

#[derive(Parser, ConsoleCommand)]
#[command(
    name = "mesh_state",
    about = "check the state of the mesh on current clip_shperer"
)]
pub struct MeshStateCommand;

pub fn check_mesh_state(
    mut mesh_state_command: ConsoleCommand<MeshStateCommand>,
    clip_spheres: Res<ClipSpheres>,
    mesh_manager: Res<MeshManager>,
    chunk_map: Res<ChunkMap>,
) {
    let mut ive3 = get_chunk_key_i3_by_vec3(clip_spheres.new_sphere.center);
    ive3.y = 0;
    let chunk_key = ChunkKey(ive3);
    println!(
        "Ready for mesh: {}",
        chunk_map.chunk_for_mesh_ready(chunk_key)
    );
    println!("Check On ChunkKey {:?}", chunk_key);
    if let Some(entity) = mesh_manager.entities.get(&chunk_key) {
        println!("Has entitiy {:?}", entity);
    } else {
        println!("Not has entitiy");
    }
    if let Some(entity) = mesh_manager.water_entities.get(&chunk_key) {
        println!("Has water entitiy {:?}", entity);
    } else {
        println!("Not has water entitiy");
    }
    if mesh_manager.fast_key.contains(&chunk_key) {
        println!("Has faste Key");
    } else {
        println!("Not has fast Key");
    }
    if let Some(state) = mesh_manager.data_status.get(&chunk_key) {
        println!("Has data state {:?}", state);
    } else {
        println!("Not has data state");
    }
    mesh_state_command.ok();
}
