use super::{
    frame_budget::FrameBudget,
    voxel_map::{NoiseConfig, Voxel, VoxelMap},
    NewSlot,
};

use building_blocks::prelude::{Array3x1, ChunkKey3, Point3f, SdfMeanDownsampler};

use bevy_tasks::{Task, TaskPool};
use futures_lite::future;
use std::time::{Duration, Instant};

pub type GenerateTaskOutput = (ChunkKey3, Option<Array3x1<Voxel>>, Duration);

pub struct ChunkGeneratorSystem {
    budget: FrameBudget,
    tasks: Vec<Task<GenerateTaskOutput>>,
}

impl ChunkGeneratorSystem {
    pub fn new(budget: FrameBudget) -> ChunkGeneratorSystem {
        ChunkGeneratorSystem {
            budget,
            tasks: Vec::new(),
        }
    }
    pub fn update(&mut self, map: &mut VoxelMap, observer_center: Point3f, noise_config: NoiseConfig, task_pool: &TaskPool, new_slots: &mut Vec<NewSlot>) {
        let mut generated_chunks = Vec::new();

        // Complete generation tasks.
        self.budget.reset_timer();
        for task in self.tasks.drain(..) {
            // PERF: is this the best way to block on many futures?
            let (chunk_key, item, item_duration) = future::block_on(task);
            self.budget.complete_item(item_duration);
            generated_chunks.push((chunk_key, item));
        }
        self.budget.update_estimate();

        // Mark chunks for loading so we can search for them asynchronously.
        for slot in new_slots.drain(..) {
            map.chunks.mark_tree_for_loading(slot.key);
        }

        // Insert generated chunks into the tree.
        {
            let span = tracing::info_span!("write_generated_chunks");
            let _trace_guard = span.enter();

            for (key, chunk) in generated_chunks.into_iter() {
                if let Some(chunk) = chunk {
                    map.chunks.write_chunk(key, chunk);
                } else {
                    // TODO: this is a temporary hack to smooth voxels; we can't delete just any "empty" chunks (those without any
                    // active edges) because there may be active edges between chunks, and the "empty" chunk might be responsible
                    // for generated the surface that intersects those edges
                    let extent = map.chunks.indexer.extent_for_chunk_with_min(key.minimum);
                    map.chunks.write_chunk(key, Array3x1::fill(extent, Voxel::EMPTY));
                    // map.chunks.delete_chunk(key);
                }
            }
        }

        // Find new chunks to load this frame.
        let mut generate_slots = Vec::new();
        let mut downsample_slots = Vec::new();
        {
            let span = tracing::info_span!("find_loading_slots");
            let _trace_guard = span.enter();

            let this_frame_budget = self.budget.request_work(0);

            map.chunks.clipmap_loading_slots(
                this_frame_budget as usize,
                false,
                observer_center,
                |key| {
                    if key.lod == 0 {
                        generate_slots.push(key);
                    } else {
                        downsample_slots.push(key);
                    }
                },
            );
        }

        // Downsample chunks. This is very fast relative to chunk generation.
        let downsampled_chunks = {
            let span = tracing::info_span!("downsample_chunks");
            let _trace_guard = span.enter();
    
            let chunks_ref = &map.chunks;
            let downsampled_chunks = task_pool.scope(|scope| {
                for dst_chunk_key in downsample_slots.drain(..) {
                    scope.spawn(async move {
                        let mut dst_chunk = chunks_ref.new_ambient_chunk(dst_chunk_key);
                        chunks_ref.downsample_children_into_external(
                            &SdfMeanDownsampler,
                            dst_chunk_key,
                            &mut dst_chunk,
                        );
    
                        (dst_chunk_key, Some(dst_chunk))
                    });
                }
            });
            downsampled_chunks
        };

        // Insert downsampled chunks into the tree.
        {
            let span = tracing::info_span!("write_downsampled_chunks");
            let _trace_guard = span.enter();

            for (key, chunk) in downsampled_chunks.into_iter() {
                if let Some(chunk) = chunk {
                    map.chunks.write_chunk(key, chunk);
                } else {
                    map.chunks.delete_chunk(key);
                }
            }
        }

        // Spawn new chunk generation tasks.
        for key in generate_slots.drain(..) {
            let chunk_extent = map.chunks.indexer.extent_for_chunk_with_min(key.minimum);
            let task = task_pool.spawn(async move {
                let span = tracing::info_span!("generate_chunk");
                let _trace_guard = span.enter();

                let start_time = Instant::now();
                let chunk = VoxelMap::generate_lod0_chunk(noise_config, chunk_extent);
                (key, chunk, start_time.elapsed())
            });
            self.tasks.push(task);
        }
    }
}
