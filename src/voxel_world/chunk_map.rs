use bevy::{
    prelude::{IVec3, Resource},
    utils::HashMap,
};
use ndshape::{ConstShape, ConstShape3u32};

use crate::{CHUNK_SIZE, CHUNK_SIZE_ADD_2_U32, CHUNK_SIZE_U32};

use super::{chunk::ChunkKey, voxel::Voxel};

#[derive(Debug, Clone, Default, Resource)]
pub struct ChunkMap {
    pub map_data: HashMap<ChunkKey, Vec<Voxel>>,
}

impl ChunkMap {
    pub fn new() -> Self {
        let data_map = HashMap::<ChunkKey, Vec<Voxel>>::new();
        Self { map_data: data_map }
    }

    pub fn get(&self, key: ChunkKey) -> Option<&Vec<Voxel>> {
        self.map_data.get(&key)
    }

    pub fn write_chunk(&mut self, chunk_key: ChunkKey, item: Vec<Voxel>) {
        self.map_data.insert(chunk_key, item);
    }

    pub fn get_by_index(volex: Option<&Vec<Voxel>>, index: u32) -> Voxel {
        match volex {
            Some(list) => list[index as usize],
            None => Voxel::EMPTY,
        }
    }

    pub fn get_with_neighbor_full_y(&mut self, chunk_key: ChunkKey) -> Vec<Voxel> {
        let mut result = Vec::new();
        type SampleShape = ConstShape3u32<CHUNK_SIZE_ADD_2_U32, 256, CHUNK_SIZE_ADD_2_U32>;
        type DataShape = ConstShape3u32<CHUNK_SIZE_ADD_2_U32, CHUNK_SIZE_U32, CHUNK_SIZE_ADD_2_U32>;
        let mut map: HashMap<i32, Vec<Voxel>> = HashMap::new();

        let last_inex = -128 / CHUNK_SIZE + 1;

        for y_offset in last_inex..=128 / CHUNK_SIZE {
            let mut new_key = chunk_key.clone();
            new_key.0.y = y_offset;
            let layer_data = self.get_layer_neighbors(new_key);
            map.insert(y_offset, layer_data);
        }

        for i in 0..SampleShape::SIZE {
            let [x, y, z] = SampleShape::delinearize(i);
            let layer = y / CHUNK_SIZE_U32;
            let layer_index: i32 = (layer as i32) + last_inex;
            let data = map.get(&(layer_index as i32));
            let index = DataShape::linearize([x, y % CHUNK_SIZE_U32, z]);
            result.push(Self::get_by_index(data, index));
        }

        result
    }

    fn get_layer_neighbors(&mut self, chunk_key: ChunkKey) -> Vec<Voxel> {
        let voxels = self.get(chunk_key);

        type SampleShape =
            ConstShape3u32<CHUNK_SIZE_ADD_2_U32, CHUNK_SIZE_U32, CHUNK_SIZE_ADD_2_U32>;
        type DataShape = ConstShape3u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32, CHUNK_SIZE_U32>;

        let px = &IVec3::new(1, 0, 0);
        let nx = &IVec3::new(-1, 0, 0);
        let pz = &IVec3::new(0, 0, 1);
        let nz = &IVec3::new(0, 0, -1);

        let offsets = vec![px, nx, pz, nz];
        let mut map: HashMap<IVec3, Vec<Voxel>> = HashMap::new();
        for ele in offsets {
            let new_key = ChunkKey(chunk_key.0 + ele.clone());
            let _ = match self.get(new_key) {
                Some(v) => {
                    map.insert(ele.clone(), v.clone());
                }
                None => (),
            };
        }
        let mut result = Vec::new();
        for i in 0..SampleShape::SIZE {
            let [x, y, z] = SampleShape::delinearize(i);
            if z != 0 && z != CHUNK_SIZE_U32 + 1 && x == CHUNK_SIZE_U32 + 1 {
                // y轴
                let index = DataShape::linearize([0, y, z - 1]);
                let v: Option<&Vec<Voxel>> = map.get(px);
                result.push(Self::get_by_index(v, index));
            } else if z != 0 && z != CHUNK_SIZE_U32 + 1 && x == 0 {
                let index = DataShape::linearize([CHUNK_SIZE_U32 - 1, y, z - 1]);
                let v = map.get(nx);
                result.push(Self::get_by_index(v, index));
            } else if x != 0 && x != CHUNK_SIZE_U32 + 1 && z == CHUNK_SIZE_U32 + 1 {
                // z轴
                let index = DataShape::linearize([x - 1, y, 0]);
                let v = map.get(pz);
                result.push(Self::get_by_index(v, index));
            } else if x != 0 && x != CHUNK_SIZE_U32 + 1 && z == 0 {
                let index = DataShape::linearize([x - 1, y, CHUNK_SIZE_U32 - 1]);
                let v = map.get(nz);
                result.push(Self::get_by_index(v, index));
            } else if x > 0 && x < CHUNK_SIZE_U32 + 1 && z > 0 && z < CHUNK_SIZE_U32 + 1 {
                let index = DataShape::linearize([x - 1, y, z - 1]);
                result.push(Self::get_by_index(voxels, index));
            } else {
                result.push(Voxel::EMPTY);
            }
        }

        result
    }
}

