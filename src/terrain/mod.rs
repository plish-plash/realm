//mod chunk_compressor;
mod chunk_generator;
mod frame_budget;
mod mesh_generator;
mod voxel_map;
pub mod voxel_mesh;

use crate::{color::Color, triangle_draw::{TriangleDraw, TriangleDrawSystem}, world::WorldSystem};
//use chunk_compressor::chunk_compression_system;
use chunk_generator::ChunkGeneratorSystem;
use mesh_generator::{MeshGeneratorSystem, MeshMaterials};
use frame_budget::FrameBudget;
use voxel_map::{MapConfig, NoiseConfig, VoxelMap};
pub use voxel_mesh::{BlockyMesh, SmoothMesh, VoxelMesh};

use building_blocks::prelude::*;
use bevy_tasks::TaskPool;

pub struct ClipSpheres {
    pub old_sphere: Sphere3,
    pub new_sphere: Sphere3,
}

impl ClipSpheres {
    pub fn new(sphere: Sphere3) -> Self {
        Self {
            old_sphere: sphere,
            new_sphere: sphere,
        }
    }
}

pub struct NewSlot {
    pub key: ChunkKey3,
}

/// System responsible for managing and updating voxel terrain
pub struct TerrainSystem<Mesh: VoxelMesh> {
    map_config: MapConfig,

    clip_spheres: ClipSpheres,
    map: VoxelMap,

    task_pool: TaskPool,
    new_slots: Vec<NewSlot>,
    chunk_generator_system: ChunkGeneratorSystem,
    mesh_generator_system: MeshGeneratorSystem<Mesh>,
}

impl<Mesh: VoxelMesh> TerrainSystem<Mesh> {
    pub fn new(draw_system: &TriangleDrawSystem) -> TerrainSystem<Mesh> {
        // TODO
        let map_config = MapConfig {
            chunk_exponent: 4,
            num_lods: 10,
            detail: 6.0,
            chunk_generation_frame_time_budget_us: 6000,
            mesh_generation_frame_time_budget_us: 6000,
            clip_radius: 500.0,
            detect_enter_lod: 4,
            noise: NoiseConfig {
                freq: 0.25,
                scale: 5.0,
                seed: 1010,
                octaves: 6,
            },
            msaa: None,
            lod_colors: false,
            wireframes: false,
        };

        let lod_colors = MeshMaterials([
            Color::rgb(1.0, 0.0, 0.0),
            Color::rgb(0.0, 1.0, 0.0),
            Color::rgb(0.0, 0.0, 1.0),
            Color::rgb(1.0, 1.0, 0.0),
            Color::rgb(0.0, 1.0, 1.0),
            Color::rgb(1.0, 0.0, 1.0),
        ].into_iter().map(|color| draw_system.load_material(color)).collect());

        let task_pool = TaskPool::new();
        let num_threads = task_pool.thread_num() as u32;
        let mut system = TerrainSystem {
            map_config,
            clip_spheres: ClipSpheres::new(Sphere3 {
                center: Point3f::fill(0.0),
                radius: map_config.clip_radius,
            }),
            map: VoxelMap::new_empty(map_config),
            task_pool,
            new_slots: Vec::new(),
            chunk_generator_system: ChunkGeneratorSystem::new(FrameBudget::new(
                num_threads,
                map_config.chunk_generation_frame_time_budget_us,
                200,
            )),
            mesh_generator_system: MeshGeneratorSystem::new(FrameBudget::new(
                num_threads,
                map_config.mesh_generation_frame_time_budget_us,
                200,
            ), lod_colors),
        };
        system.setup();
        system
    }

    fn setup(&mut self) {
        // Detect the initial set of slots inside the clip sphere.
        let indexer = self.map.chunks.indexer.clone();
        let mut new_slots = Vec::new();
        clipmap_chunks_intersecting_sphere(
            &indexer,
            self.map_config.root_lod(),
            self.map_config.detect_enter_lod,
            self.map_config.detail,
            self.clip_spheres.new_sphere,
            |new_slot| new_slots.push(new_slot),
        );
        self.new_slots.extend(new_slots.into_iter().map(|s| NewSlot { key: s.key }));
    }

    pub fn update_clip_spheres(&mut self, camera_position: cgmath::Point3<f32>) {
        self.clip_spheres.old_sphere = self.clip_spheres.new_sphere;
        self.clip_spheres.new_sphere = Sphere3 {
            center: Point3f::from(camera_position),
            radius: self.map_config.clip_radius,
        };
    }
    fn detect_new_slots(&mut self) {
        let span = tracing::info_span!("clip_events");
        let _trace_guard = span.enter();

        let indexer = self.map.chunks.indexer.clone();
        let mut new_slots = Vec::new();
        clipmap_new_chunks_intersecting_sphere(
            &indexer,
            self.map_config.root_lod(),
            self.map_config.detect_enter_lod,
            self.map_config.detail,
            self.clip_spheres.old_sphere,
            self.clip_spheres.new_sphere,
            |new_slot| new_slots.push(new_slot),
        );
        self.new_slots.extend(new_slots.into_iter().map(|s| NewSlot { key: s.key }));
    }
}

impl<Mesh: VoxelMesh> WorldSystem for TerrainSystem<Mesh> {
    fn update(&mut self, draw_system: &TriangleDrawSystem, _delta_time: f64) {
        self.detect_new_slots();

        // generate chunks
        self.chunk_generator_system.update(&mut self.map, self.clip_spheres.new_sphere.center, self.map_config.noise, &self.task_pool, &mut self.new_slots);

        // generate meshes
        self.mesh_generator_system.update(draw_system, &self.map, &self.clip_spheres, &self.task_pool);

        // self.compress_chunks();
    }

    fn render(&self, renderer: &mut TriangleDraw) {
        self.mesh_generator_system.render(renderer);
    }
}
