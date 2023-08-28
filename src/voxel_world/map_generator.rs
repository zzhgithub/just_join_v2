use ndshape::{ConstShape, ConstShape2u32, ConstShape3u32};
#[cfg(target_arch = "aarch64")]
use noise::utils::NoiseMapBuilder;
#[cfg(target_arch = "x86_64")]
use simdnoise::NoiseBuilder;

use crate::{
    voxel_world::voxel::{BasicStone, Grass, Sand, Soli, Sown, Stone, VoxelMaterial, Water},
    CHUNK_SIZE, CHUNK_SIZE_U32,
};

use super::{chunk::ChunkKey, voxel::Voxel};

pub fn gen_chunk_data_by_seed(seed: i32, chunk_key: ChunkKey) -> Vec<Voxel> {
    // let base_x = (chunk_key.0.x * CHUNK_SIZE) as f32;
    let base_y: f32 = (chunk_key.0.y * CHUNK_SIZE) as f32;
    // let base_z = (chunk_key.0.z * CHUNK_SIZE) as f32;
    type SampleShape = ConstShape3u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32, CHUNK_SIZE_U32>;
    type PanleShap = ConstShape2u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32>;
    let mut voxels = Vec::new();

    let noise = noise2d(chunk_key, seed);
    let noise2 = noise2d_ridge(chunk_key, seed);

    for i in 0..SampleShape::SIZE {
        let [x, y, z] = SampleShape::delinearize(i);
        // let p_x = base_x + x as f32;
        let p_y = base_y + y as f32;
        // let p_z = base_z + z as f32;

        let h = -60.;
        // println!("({},{})", h, p_y);
        let index = PanleShap::linearize([x, z]);
        let top = h + fn_height(noise[index as usize]) + noise2[index as usize] * 5.0;
        if p_y <= top {
            if p_y >= -60. + 110. {
                voxels.push(Sown::into_voxel());
                continue;
            }
            if p_y <= -110. {
                voxels.push(BasicStone::into_voxel());
                continue;
            }
            if p_y >= -60. + 100. {
                voxels.push(Stone::into_voxel());
                continue;
            }
            if p_y >= top - 1.0 {
                if p_y < -60. + 76. {
                    voxels.push(Soli::into_voxel());
                } else {
                    voxels.push(Grass::into_voxel());
                }
            } else if p_y > top - 5.0 {
                voxels.push(Soli::into_voxel());
            } else {
                voxels.push(Stone::into_voxel());
            }
        } else {
            voxels.push(Voxel::EMPTY);
        }
    }
    // 海平面 TODO: 更加优秀的还平面
    let mut water_flag = false;
    for i in 0..SampleShape::SIZE {
        let [_, y, _] = SampleShape::delinearize(i);
        let p_y: f32 = base_y + y as f32;
        if p_y <= -60. + 76. && voxels[i as usize].id == Voxel::EMPTY.id {
            water_flag = true;
            voxels[i as usize] = Water::into_voxel();
        }
    }

    //生成 沙子
    if water_flag {
        for i in 0..SampleShape::SIZE {
            let [x, y, z] = SampleShape::delinearize(i);
            if (check_water(voxels.clone(), [x + 1, y, z])
                || (x != 0 && check_water(voxels.clone(), [x - 1, y, z]))
                || check_water(voxels.clone(), [x, y + 1, z])
                || (y != 0 && check_water(voxels.clone(), [x, y - 1, z]))
                || check_water(voxels.clone(), [x, y, z + 1])
                || (z != 0 && check_water(voxels.clone(), [x, y, z - 1])))
                && voxels[i as usize].id != Water::ID
                && voxels[i as usize].id != Voxel::EMPTY.id
            {
                voxels[i as usize] = Sand::into_voxel()
            }
        }
    }

    //侵蚀 洞穴
    let noise_3d = noise3d_2(chunk_key, seed);
    for i in 0..SampleShape::SIZE {
        // let [x, y, z] = SampleShape::delinearize(i);
        // let index = SampleShape::linearize([x, z, y]);
        let flag: f32 = noise_3d[i as usize];
        if flag < 0.05
            && flag > -0.05
            && voxels[i as usize].id != Water::ID
            && voxels[i as usize].id != BasicStone::ID
        {
            voxels[i as usize] = Voxel::EMPTY;
        }
    }

    voxels
}

pub fn check_water(voxels: Vec<Voxel>, point: [u32; 3]) -> bool {
    type SampleShape = ConstShape3u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32, CHUNK_SIZE_U32>;
    let index = SampleShape::linearize(point);
    if point[0] >= CHUNK_SIZE_U32 || point[1] >= CHUNK_SIZE_U32 || point[2] >= CHUNK_SIZE_U32 {
        return false;
    }

    voxels[index as usize].id == Water::ID
}

#[cfg(target_arch = "aarch64")]
pub fn noise2d(chunk_key: ChunkKey, seed: i32) -> Vec<f32> {
    let mut noise = noise::Fbm::<noise::SuperSimplex>::new(seed as u32);
    noise.octaves = 4;
    noise.frequency = 0.005;
    noise.persistence = 0.5;
    noise.lacunarity = 2.0;
    let x_offset = (chunk_key.0.x * CHUNK_SIZE) as f64;
    let z_offset = (chunk_key.0.z * CHUNK_SIZE) as f64;

    noise::utils::PlaneMapBuilder::<_, 2>::new(noise)
        .set_size(CHUNK_SIZE as usize, CHUNK_SIZE as usize)
        .set_x_bounds(x_offset, x_offset + CHUNK_SIZE as f64)
        .set_y_bounds(z_offset, z_offset + CHUNK_SIZE as f64)
        .build()
        .into_iter()
        .map(|x| x.mul_add(20f64, 132f64) as f32)
        .collect()
}

// 生成2d的柏林噪声
#[cfg(target_arch = "x86_64")]
pub fn noise2d(chunk_key: ChunkKey, seed: i32) -> Vec<f32> {
    let (noise, _max, _min) = NoiseBuilder::fbm_2d_offset(
        (chunk_key.0.x * CHUNK_SIZE) as f32,
        CHUNK_SIZE as usize,
        (chunk_key.0.z * CHUNK_SIZE) as f32,
        CHUNK_SIZE as usize,
    )
    .with_seed(seed)
    .with_freq(0.05)
    .with_octaves(4)
    .generate();
    noise
}

#[cfg(target_arch = "aarch64")]
pub fn noise2d_ridge(chunk_key: ChunkKey, seed: i32) -> Vec<f32> {
    let mut noise = noise::Fbm::<noise::RidgedMulti<noise::Perlin>>::new(seed as u32);
    noise.octaves = 6;
    noise.frequency = 0.003;
    noise.persistence = 0.5;
    noise.lacunarity = 2.0;
    let x_offset = (chunk_key.0.x * CHUNK_SIZE) as f64;
    let z_offset = (chunk_key.0.z * CHUNK_SIZE) as f64;

    noise::utils::PlaneMapBuilder::<_, 2>::new(noise)
        .set_size(CHUNK_SIZE as usize, CHUNK_SIZE as usize)
        .set_x_bounds(x_offset, x_offset + CHUNK_SIZE as f64)
        .set_y_bounds(z_offset, z_offset + CHUNK_SIZE as f64)
        .build()
        .into_iter()
        .map(|x| x.mul_add(1f64, 0f64) as f32)
        .collect()
}

#[cfg(target_arch = "x86_64")]
pub fn noise2d_ridge(chunk_key: ChunkKey, seed: i32) -> Vec<f32> {
    let (noise, _, _) = NoiseBuilder::ridge_2d_offset(
        (chunk_key.0.x * CHUNK_SIZE) as f32,
        CHUNK_SIZE as usize,
        (chunk_key.0.z * CHUNK_SIZE) as f32,
        CHUNK_SIZE as usize,
    )
    .with_seed(seed)
    .with_freq(0.03)
    .with_octaves(5)
    .with_gain(4.0)
    .with_lacunarity(0.5)
    .generate();
    noise
}

#[cfg(target_arch = "aarch64")]
// 尝试产生 洞穴的噪声
pub fn noise3d_2(chunk_key: ChunkKey, seed: i32) -> Vec<f32> {
    use noise::NoiseFn;

    let mut fbm: noise::Fbm<noise::Perlin> = noise::Fbm::<noise::Perlin>::new(seed as u32);
    fbm.octaves = 6;
    fbm.frequency = 0.2;
    fbm.persistence = 2.0;
    fbm.lacunarity = 0.5;
    let x_offset = (chunk_key.0.x * CHUNK_SIZE) as f64;
    let z_offset = (chunk_key.0.z * CHUNK_SIZE) as f64;
    let y_offset = (chunk_key.0.y * CHUNK_SIZE) as f64;

    let mut noise: Vec<f32> = Vec::new();
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let pos = [
                    x_offset + x as f64,
                    z_offset + z as f64,
                    y_offset + y as f64,
                ];
                noise.push(fbm.get(pos) as f32 / 10.);
            }
        }
    }
    noise
}

#[cfg(target_arch = "x86_64")]
// 尝试产生 洞穴的噪声
pub fn noise3d_2(chunk_key: ChunkKey, seed: i32) -> Vec<f32> {
    let (noise, _, _) = NoiseBuilder::fbm_3d_offset(
        (chunk_key.0.x * CHUNK_SIZE) as f32,
        CHUNK_SIZE as usize,
        (chunk_key.0.y * CHUNK_SIZE) as f32,
        CHUNK_SIZE as usize,
        (chunk_key.0.z * CHUNK_SIZE) as f32,
        CHUNK_SIZE as usize,
    )
    .with_seed(seed)
    .with_freq(0.2)
    .with_lacunarity(0.5)
    .with_gain(2.0)
    .with_octaves(6)
    .generate();
    noise
}

#[cfg(target_arch = "aarch64")]
pub fn fn_height(x: f32) -> f32 {
    x - 40.
}

#[cfg(target_arch = "x86_64")]
// 对数据进行差值处理
pub fn fn_height(x: f32) -> f32 {
    if x < -0.6 {
        // print!("a{}", x);
        return 60.;
    }
    if x >= -0.6 && x < -0.5 {
        // print!("b{}", x);
        return 60. + 150. * (x - 0.6);
    }
    if x >= -0.5 && x < 0.0 {
        // print!("c{}", x);
        return 75.;
    }
    if x >= 0.0 && x < 0.1 {
        return 75. + 100. * x;
    }
    if x >= 0.1 && x < 0.2 {
        return 85. + 150. * (x - 0.1);
    }
    if x >= 0.2 {
        return 100. + 100. * (x - 0.2);
    }
    0.
}
