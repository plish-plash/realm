use crate::triangle_draw::{TriangleDraw, TriangleDrawSystem, TriangleDrawable, TriangleMaterialHandle};
use crate::transform::{Transform, TransformExtensions};

use super::{
    frame_budget::FrameBudget,
    voxel_map::VoxelMap,
    voxel_mesh::VoxelMesh,
    ClipSpheres,
};

use bevy_tasks::{TaskPool, Task};
use building_blocks::{mesh::*, prelude::*, storage::SmallKeyHashMap};

use futures_lite::future;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

/// Starts and polls tasks to generate new meshes.
///
/// In order to have tasks running for the full duration of a frame, we first poll all outstanding tasks to completion, then
/// spawn new ones.
pub struct MeshGeneratorSystem<Mesh: VoxelMesh> {
    _marker: PhantomData<Mesh>,
    budget: FrameBudget,
    chunk_meshes: ChunkMeshes,
    
    mesh_tasks: MeshTasks,
    mesh_materials: MeshMaterials,
}

impl<Mesh: VoxelMesh> MeshGeneratorSystem<Mesh> {
    pub fn new(budget: FrameBudget, mesh_materials: MeshMaterials) -> MeshGeneratorSystem<Mesh> {
        MeshGeneratorSystem {
            _marker: PhantomData,
            budget,
            chunk_meshes: ChunkMeshes::default(),
            mesh_tasks: MeshTasks::default(),
            mesh_materials,
        }
    }

    fn create_and_destroy_meshes(&mut self, draw_system: &TriangleDrawSystem) {
        self.budget.reset_timer();
    
        // Finish all outstanding tasks.
        for task in self.mesh_tasks.tasks.drain(..) {
            // PERF: is this the best way to block on many futures?
            let (chunk_key, item, item_duration) = future::block_on(task);
    
            self.budget.complete_item(item_duration);
    
            if let Some(mesh) = item {
                self.chunk_meshes.entities.insert(
                    chunk_key,
                    TriangleDrawable {
                        mesh: draw_system.load_mesh(mesh),
                        material: self.mesh_materials.0[chunk_key.lod as usize].clone(),
                        transform: Transform::identity(),
                    },
                );
            } else {
                self.chunk_meshes.entities.remove(&chunk_key);
            }
        }
    
        self.budget.update_estimate();
    
        for chunk_key in self.mesh_tasks.removals.drain(..) {
            self.chunk_meshes.entities.remove(&chunk_key);
        }
    }

    fn start_mesh_tasks(&mut self, voxel_map: &VoxelMap, task_pool: &TaskPool, updates: Vec<LodChange3>) {
        let MeshTasks { tasks, removals } = &mut self.mesh_tasks;
    
        let mut start_task = |key: ChunkKey3| {
            if voxel_map.chunks.get_chunk(key).is_none() {
                return;
            }
    
            let chunk_shape = voxel_map.chunks.chunk_shape();
            let neighborhood = Mesh::copy_chunk_neighborhood(&voxel_map.chunks, key);
    
            let task = task_pool.spawn(async move {
                let span = tracing::info_span!("make_mesh");
                let _trace_guard = span.enter();
    
                let start_time = Instant::now();
    
                let mut mesh_buffers = Mesh::init_mesh_buffers(chunk_shape);
    
                let mesh = Mesh::create_mesh_for_chunk(key, &neighborhood, &mut mesh_buffers);
    
                (key, mesh, start_time.elapsed())
            });
            tasks.push(task);
        };
    
        for update in updates.into_iter() {
            match update {
                LodChange3::Spawn(key) => {
                    start_task(key);
                }
                LodChange3::Split(split) => {
                    removals.push(split.old_chunk);
                    for &key in split.new_chunks.iter() {
                        start_task(key);
                    }
                }
                LodChange3::Merge(merge) => {
                    for &key in merge.old_chunks.iter() {
                        removals.push(key);
                    }
                    start_task(merge.new_chunk);
                }
            }
        }
    }

    pub fn update(&mut self, draw_system: &TriangleDrawSystem, map: &VoxelMap, clip_spheres: &ClipSpheres, task_pool: &TaskPool) {
        self.cleanup_out_of_range(map.chunks.indexer.clone(), clip_spheres);
        self.create_and_destroy_meshes(draw_system);
    
        // Find render updates.
        let mut updates = Vec::new();
        let span = tracing::info_span!("lod_changes");
        {
            let _trace_guard = span.enter();
    
            let this_frame_budget = self.budget.request_work(0);
    
            map.chunks.clipmap_render_updates(
                map.config.detail,
                clip_spheres.new_sphere.center,
                this_frame_budget as usize,
                |c| updates.push(c),
            );
        }
    
        self.start_mesh_tasks(map, task_pool, updates);
    }
    
    /// Deletes meshes that aren't bounded by the clip sphere.
    fn cleanup_out_of_range(&mut self, indexer: ChunkIndexer3, clip_spheres: &ClipSpheres) {
        let mut chunks_to_remove = Vec::new();
        for &chunk_key in self.chunk_meshes.entities.keys() {
            let chunk_sphere = chunk_bounding_sphere(&indexer, chunk_key, 0);
            if !clip_spheres.new_sphere.intersects(&chunk_sphere) {
                chunks_to_remove.push(chunk_key);
            }
        }

        for chunk_key in chunks_to_remove.into_iter() {
            self.chunk_meshes.entities.remove(&chunk_key);
        }
    }

    pub fn render(&self, renderer: &mut TriangleDraw) {
        for mesh in self.chunk_meshes.entities.values() {
            renderer.draw(mesh);
        }
    }
}

pub struct MeshMaterials(pub Vec<TriangleMaterialHandle>);

#[derive(Default)]
pub struct ChunkMeshes {
    // Map from chunk key to mesh entity.
    entities: SmallKeyHashMap<ChunkKey3, TriangleDrawable>,
}

/// All mesh tasks currently running.
#[derive(Default)]
pub struct MeshTasks {
    tasks: Vec<Task<MeshTaskOutput>>,
    // These need to be applied in the same frame when the new meshes are created so splits/merges happen atomically.
    removals: Vec<ChunkKey3>,
}

pub type MeshTaskOutput = (ChunkKey3, Option<PosNormMesh>, Duration);
