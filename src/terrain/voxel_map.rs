use building_blocks::{
    core::bytemuck::{Pod, Zeroable},
    mesh::{IsOpaque, MergeVoxel},
    prelude::*,
};
use simdnoise::NoiseBuilder;

pub fn noise_array3(extent: Extent3i, freq: f32, seed: i32, octaves: u8) -> Array3x1<f32> {
    let min = Point3f::from(extent.minimum);
    let (noise, _min_val, _max_val) = NoiseBuilder::fbm_3d_offset(
        min.x(),
        extent.shape.x() as usize,
        min.y(),
        extent.shape.y() as usize,
        min.z(),
        extent.shape.z() as usize,
    )
    .with_freq(freq)
    .with_seed(seed)
    .with_octaves(octaves)
    .generate();

    Array3x1::new_one_channel(extent, noise.into_boxed_slice())
}

pub fn generate_noise_chunk3(
    extent: Extent3i,
    freq: f32,
    scale: f32,
    seed: i32,
    octaves: u8,
    subsurface_only: bool,
) -> Option<Array3x1<f32>> {
    let mut array = noise_array3(extent, freq, seed, octaves);

    if subsurface_only {
        let mut any_negative = false;
        array.for_each_mut(&extent, |_: (), x| {
            *x *= scale;

            if *x < 0.0 {
                any_negative = true;
            }
        });

        if any_negative {
            Some(array)
        } else {
            None
        }
    } else {
        array.for_each_mut(&extent, |_: (), x| {
            *x *= scale;
        });

        Some(array)
    }
}

pub struct VoxelMap {
    pub config: MapConfig,
    pub chunks: CompressibleChunkTree3x1<Lz4, Voxel>,
}

impl VoxelMap {
    pub fn new_empty(config: MapConfig) -> Self {
        let chunk_shape = config.chunk_shape();
        let root_lod = config.num_lods - 1;
        let builder = ChunkTreeBuilder3x1::new(ChunkTreeConfig {
            chunk_shape,
            ambient_value: Voxel::EMPTY,
            root_lod,
        });
        let chunks = builder.build_with_storage(|| {
            FastCompressibleChunkStorageNx1::with_bytes_compression(Lz4 { level: 10 })
        });

        Self { chunks, config }
    }

    pub fn generate_lod0_chunk(
        config: NoiseConfig,
        chunk_extent: Extent3i,
    ) -> Option<Array3x1<Voxel>> {
        let NoiseConfig {
            freq,
            scale,
            seed,
            octaves,
        } = config;

        unsafe {
            // SAFE: Voxel is a transparent wrapper of f32
            std::mem::transmute(generate_noise_chunk3(
                chunk_extent,
                freq,
                scale,
                seed,
                octaves,
                true,
            ))
        }
    }
}

#[derive(Clone, Copy)]
pub struct MapConfig {
    pub chunk_exponent: u8,
    pub num_lods: u8,
    pub clip_radius: f32,
    pub detect_enter_lod: u8,
    pub detail: f32,
    pub chunk_generation_frame_time_budget_us: u32,
    pub mesh_generation_frame_time_budget_us: u32,
    pub noise: NoiseConfig,
    pub wireframes: bool,
    pub lod_colors: bool,
    pub msaa: Option<u32>,
}

impl MapConfig {
    pub fn chunk_shape(&self) -> Point3i {
        Point3i::fill(1 << self.chunk_exponent)
    }
    pub fn root_lod(&self) -> u8 {
        self.num_lods - 1
    }
}

#[derive(Clone, Copy)]
pub struct NoiseConfig {
    pub freq: f32,
    pub scale: f32,
    pub seed: i32,
    pub octaves: u8,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Voxel(pub f32);

impl Voxel {
    pub const EMPTY: Self = Self(1.0);
    pub const FILLED: Self = Self(-1.0);
}

unsafe impl Zeroable for Voxel {}
unsafe impl Pod for Voxel {}

impl IsEmpty for Voxel {
    fn is_empty(&self) -> bool {
        self.0 >= 0.0
    }
}

impl IsOpaque for Voxel {
    fn is_opaque(&self) -> bool {
        true
    }
}

impl MergeVoxel for Voxel {
    type VoxelValue = bool;

    fn voxel_merge_value(&self) -> Self::VoxelValue {
        self.0 < 0.0
    }
}

impl From<Voxel> for f32 {
    fn from(v: Voxel) -> Self {
        v.0
    }
}

impl From<f32> for Voxel {
    fn from(x: f32) -> Self {
        Voxel(x)
    }
}

impl SignedDistance for Voxel {
    fn is_negative(&self) -> bool {
        self.0.is_negative()
    }
}
