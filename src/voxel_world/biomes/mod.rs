use bevy::{
    prelude::{IVec3, Plugin, ResMut, Resource, Update, Vec3},
    tasks::AsyncComputeTaskPool,
    utils::{HashMap, HashSet},
};
use ndshape::{ConstShape, ConstShape2u32, ConstShape3u32};
use noise::{
    core::worley::{distance_functions::euclidean, ReturnType},
    utils::NoiseMapBuilder,
    Worley,
};

use crate::{
    server::{async_chunk::ChunkResultTasks, message_def::chunk_result::ChunkResult},
    tools::chunk_key_any_xyz_to_vec3,
    CHUNK_SIZE, CHUNK_SIZE_U32,
};

use self::{
    basic_land::BasicLandBiomes,
    bule_land::BuleLandBoimes,
    dry_land::DryLandBiomes,
    sand_land::SandLandBiomes,
    sdf::{sd_cut_sphere, trunk},
    snow_land::SnowLandBiomes,
};

use super::{
    chunk::ChunkKey, chunk_map::ChunkMap, compress::compress, map_database::DbSaveTasks,
    voxel::Voxel,
};

pub mod basic_land;
pub mod bule_land;
pub mod dry_land;
pub mod sand_land;
pub mod sdf;
pub mod snow_land;

pub type SampleShape = ConstShape3u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32, CHUNK_SIZE_U32>;
pub type PanleShap = ConstShape2u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32>;

// 处理 生物群落
pub fn biomes_generate(
    chunk_key: ChunkKey,
    seed: i32,
    suface_index: Vec<u32>,
    voxels: &mut Vec<Voxel>,
) -> Vec<(Vec<ChunkKey>, TreeGentor)> {
    let mut ret = Vec::new();
    if suface_index.len() == 0 {
        return ret;
    }
    // 生成噪声
    let noise = biomes_noise(chunk_key, seed);
    // 这里产生一个 种树的噪声
    let tree_noise = tree_noise(chunk_key, seed);

    for index in suface_index {
        // 由噪声生产的特征值
        let [x, _, z] = SampleShape::delinearize(index);
        let index_2d = PanleShap::linearize([x, z]);
        let atrr = noise[index_2d as usize];
        let generator = get_generator_by_atrr(atrr);
        generator.gen_land(chunk_key.clone(), voxels, index, index_2d);
        // fixme: 这里要记录对于其他方块的影响
        if tree_noise[index_2d as usize] > 0.99 {
            if let Some(rs) = generator.make_tree(chunk_key.clone(), voxels, index, index_2d) {
                ret.push(rs);
            }
        }
    }
    ret
}

// 获取不同的生成器
fn get_generator_by_atrr(data: f32) -> Box<dyn BiomesGenerator> {
    if data < 0.1 {
        return BasicLandBiomes.into_boxed_generator();
    } else if data < 0.4 {
        return DryLandBiomes.into_boxed_generator();
    } else if data < 0.6 {
        return SnowLandBiomes.into_boxed_generator();
    } else if data < 0.8 {
        return SandLandBiomes.into_boxed_generator();
    } else {
        return BuleLandBoimes.into_boxed_generator();
    }
}

pub fn tree_noise(chunk_key: ChunkKey, seed: i32) -> Vec<f32> {
    let noise = Worley::new(seed as u32)
        .set_distance_function(euclidean)
        .set_return_type(ReturnType::Value)
        .set_frequency(1.0);

    let x_offset = (chunk_key.0.x * CHUNK_SIZE) as f64;
    let z_offset = (chunk_key.0.z * CHUNK_SIZE) as f64;

    noise::utils::PlaneMapBuilder::<_, 2>::new(noise)
        .set_size(CHUNK_SIZE as usize, CHUNK_SIZE as usize)
        .set_x_bounds(x_offset, x_offset + CHUNK_SIZE as f64)
        .set_y_bounds(z_offset, z_offset + CHUNK_SIZE as f64)
        .build()
        .into_iter()
        .map(|x| x as f32)
        .collect()
}

pub fn biomes_noise(chunk_key: ChunkKey, seed: i32) -> Vec<f32> {
    let noise = Worley::new(seed as u32)
        .set_distance_function(euclidean)
        .set_return_type(ReturnType::Value)
        .set_frequency(0.008);

    let x_offset = (chunk_key.0.x * CHUNK_SIZE) as f64;
    let z_offset = (chunk_key.0.z * CHUNK_SIZE) as f64;

    noise::utils::PlaneMapBuilder::<_, 2>::new(noise)
        .set_size(CHUNK_SIZE as usize, CHUNK_SIZE as usize)
        .set_x_bounds(x_offset, x_offset + CHUNK_SIZE as f64)
        .set_y_bounds(z_offset, z_offset + CHUNK_SIZE as f64)
        .build()
        .into_iter()
        .map(|x| x as f32)
        .collect()
}

pub trait BiomesGenerator: 'static + Sync + Send {
    fn gen_land_with_info(
        &self,
        chunk_key: ChunkKey,
        voxels: &mut Vec<Voxel>,
        chunk_index: u32,
        plane_index: u32,
        height: f32,
        xyz: [u32; 3],
    );

    fn gen_land(
        &self,
        chunk_key: ChunkKey,
        voxels: &mut Vec<Voxel>,
        chunk_index: u32,
        plane_index: u32,
    ) {
        let base_y: f32 = (chunk_key.0.y * CHUNK_SIZE) as f32;
        let [x, y, z] = SampleShape::delinearize(chunk_index);
        let height = base_y + y as f32;
        self.gen_land_with_info(
            chunk_key,
            voxels,
            chunk_index,
            plane_index,
            height,
            [x, y, z],
        );
    }

    fn make_tree(
        &self,
        chunk_key: ChunkKey,
        voxels: &mut Vec<Voxel>,
        chunk_index: u32,
        plane_index: u32,
    ) -> Option<(Vec<ChunkKey>, TreeGentor)> {
        let base_y: f32 = (chunk_key.0.y * CHUNK_SIZE) as f32;
        let [x, y, z] = SampleShape::delinearize(chunk_index);
        let height = base_y + y as f32;
        self.make_tree_with_info(
            chunk_key,
            voxels,
            chunk_index,
            plane_index,
            height,
            [x, y, z],
        )
    }

    fn make_tree_with_info(
        &self,
        _chunk_key: ChunkKey,
        _voxels: &mut Vec<Voxel>,
        _chunk_index: u32,
        _plane_index: u32,
        _height: f32,
        _xyz: [u32; 3],
    ) -> Option<(Vec<ChunkKey>, TreeGentor)> {
        // do nothing;
        None
    }
}

pub trait IntoBoxedTerrainGenerator: BiomesGenerator + Sized {
    fn into_boxed_generator(self) -> Box<Self>;
}

impl<T: BiomesGenerator> IntoBoxedTerrainGenerator for T {
    fn into_boxed_generator(self) -> Box<Self> {
        Box::new(self)
    }
}
// 海平面
pub const SEE_LEVEL: f32 = -60. + 76.;
// 山峰线
pub const MOUNTAIN_LEVEL: f32 = -60. + 100.;
// 雪线
pub const SNOW_LEVEL: f32 = -60. + 110.;

#[derive(Debug, Clone)]
pub struct TreeGentor {
    pub tree: Voxel,
    pub leaf: Voxel,
    pub trunk_params: (Vec3, u32),
    pub leafs_params: (Vec3, f32, f32),
}

impl TreeGentor {
    pub fn make_tree_for_chunk(&mut self, voxels: &mut Vec<Voxel>, chunk_key: ChunkKey) {
        let mut trunk_fn = trunk(self.trunk_params.0, self.trunk_params.1);
        let mut leaf_fn = sd_cut_sphere(
            self.leafs_params.0,
            self.leafs_params.1,
            self.leafs_params.2,
        );

        for index in 0..SampleShape::SIZE {
            let inner_xyz = SampleShape::delinearize(index);
            let check_pos = chunk_key_any_xyz_to_vec3(chunk_key, inner_xyz);
            if trunk_fn(check_pos.clone()) <= 0.0 {
                voxels[index as usize] = self.tree.clone();
            } else if leaf_fn(check_pos.clone()) <= 0.0
                && voxels[index as usize].id == Voxel::EMPTY.id
            {
                voxels[index as usize] = self.leaf.clone();
            }
        }
    }
}

pub fn find_out_chunk_keys(xyz: [u32; 3], chunk_key: ChunkKey, h: u32, r: u32) -> Vec<ChunkKey> {
    let mut ret = Vec::new();
    let [x, y, z] = xyz;
    let mut x_offset = 0;
    let mut z_offset = 0;
    if x < r {
        ret.push(chunk_key.add_ivec3(IVec3::new(-1, 0, 0)));
        x_offset = -1;
    } else if x + r >= CHUNK_SIZE_U32 {
        ret.push(chunk_key.add_ivec3(IVec3::new(1, 0, 0)));
        x_offset = -1;
    }
    if z < r {
        ret.push(chunk_key.add_ivec3(IVec3::new(0, 0, -1)));
        z_offset = -1;
    } else if z + r >= CHUNK_SIZE_U32 {
        ret.push(chunk_key.add_ivec3(IVec3::new(0, 0, 1)));
        z_offset = 1;
    }
    let mut y_offset = 0;
    if y + h - 1 + r >= CHUNK_SIZE_U32 {
        ret.push(chunk_key.add_ivec3(IVec3::new(0, 1, 0)));
        y_offset = 1;
    }

    if x_offset != 0 && z_offset != 0 {
        ret.push(chunk_key.add_ivec3(IVec3::new(x_offset, 0, z_offset)));
    }
    if y_offset != 0 {
        if x_offset != 0 {
            ret.push(chunk_key.add_ivec3(IVec3::new(x_offset, y_offset, 0)));
        }
        if z_offset != 0 {
            ret.push(chunk_key.add_ivec3(IVec3::new(0, y_offset, z_offset)));
        }
        if x_offset != 0 && z_offset != 0 {
            ret.push(chunk_key.add_ivec3(IVec3::new(x_offset, y_offset, z_offset)));
        }
    }

    ret
}

#[derive(Resource)]
pub struct OtherTreeTasksMap {
    pub tree_map: HashMap<ChunkKey, Vec<TreeGentor>>,
}

impl OtherTreeTasksMap {
    pub fn insert(&mut self, list: Vec<(Vec<ChunkKey>, TreeGentor)>) {
        for (keys, tree_gentor) in list.into_iter() {
            for key in keys.iter() {
                self.tree_map
                    .entry(*key)
                    .or_insert(Vec::new())
                    .push(tree_gentor.clone());
            }
        }
    }
}

pub struct OtherTreePlugin;

impl Plugin for OtherTreePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(OtherTreeTasksMap {
            tree_map: HashMap::new(),
        });
        app.add_systems(Update, deal_other_tree);
    }
}

fn deal_other_tree(
    mut db_save_task: ResMut<DbSaveTasks>,
    mut chunk_map: ResMut<ChunkMap>,
    mut other_tree_tasks_map: ResMut<OtherTreeTasksMap>,
    mut tasks: ResMut<ChunkResultTasks>,
) {
    let pool = AsyncComputeTaskPool::get();
    let mut exit_keys: HashSet<ChunkKey> = HashSet::new();

    for (chunk_key, list) in other_tree_tasks_map.tree_map.iter_mut() {
        if let Some(voxels) = chunk_map.map_data.get_mut(&chunk_key.clone()) {
            for tree_gentor in list {
                tree_gentor.make_tree_for_chunk(voxels, chunk_key.clone());
                exit_keys.insert(chunk_key.clone());
            }
        }
    }

    for key in exit_keys {
        if let Some(_) = other_tree_tasks_map.tree_map.remove(&key) {
            if let Some(data) = chunk_map.map_data.get(&key) {
                let voxels = data.clone();
                let (buffer, tree) = compress(voxels.clone());
                let message = if buffer.len() == 0 {
                    bincode::serialize(&ChunkResult::ChunkSame((key, voxels[0]))).unwrap()
                } else {
                    bincode::serialize(&ChunkResult::UpdateChunkData {
                        key: key,
                        data: (buffer, tree),
                    })
                    .unwrap()
                };

                let task = pool.spawn(async move { (0, message) });
                tasks.tasks.push(task);

                let task = pool.spawn(async move { (key.as_u8_array(), voxels.clone()) });
                db_save_task.tasks.push(task);
            }
        }
    }
}
