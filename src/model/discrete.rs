use array_init::array_init;
use itertools;
use itertools::Itertools;
use lazy_init::Lazy;
use rayon::prelude::*;
use std::collections::HashSet;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SendError;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

use crate::config;
use crate::config::WORLD_SIZE;
use crate::get_minecraft_chunk_position;
use crate::minecraft;
use crate::model::implicit::smooth::polygonize_chunk;
use crate::time_it;

use super::chunk::{BlockData, Chunk, ChunkPosition};
use super::common::BlockType;
use super::polygonize::Mesh;
use super::polygonize::PolygonizationOptions;
use super::Position;

const CHUNKS_IN_WORLD: usize = WORLD_SIZE * WORLD_SIZE;

pub type WorldChunks = [Arc<Chunk>; CHUNKS_IN_WORLD];

fn clone_world(chunks: &WorldChunks) -> WorldChunks {
    let clone = array_init(|index| chunks[index].clone());
    clone
}

// A mesh of a chunk located at *ChunkPosition*
struct BoundMesh(Mesh, ChunkPosition);

enum ChunkSource {
    Direct(Chunk),
    Reference(usize),
}

struct ChunkChange(usize, ChunkSource);
struct WorldChange(ChunkPosition, JoinHandle<Vec<ChunkChange>>);

// Represents a 2D grid of chunks
// Rows are parallel to the world x axis
// Columns are parallel to the world z axis
//
// Bigger indices correspond to bigger coordinates
//
// At any time only a part of the world is loaded, see config::WORLD_SIZE
// This strusts represents a "window" into the the world
// and can be used as a sliding window centered around the player
pub struct World {
    // Internal grid representation as a flat array
    chunks: WorldChunks,

    // Position of the center chunk in the world
    center: ChunkPosition,

    // Meshes are built in parallel in another thread.
    // We use channels to send the built meshes back and they are then integrated into
    // the world in the update loop. Is this needlessly complicated?
    // NO!
    // When moving diagonal to the chunk grid, we need to load meshes for chunks is rapid
    // succession. That is why multiple we need support for multiple concurrent updates.
    chunk_meshes: [Lazy<Mesh>; CHUNKS_IN_WORLD],
    mesh_sender: Sender<BoundMesh>,
    mesh_receiver: Receiver<BoundMesh>,
    mesh_builders: Vec<JoinHandle<Vec<SendError<BoundMesh>>>>,
    meshes_being_built: HashSet<ChunkPosition>,

    // Handle to the worker thread that loads chunks from minecraft save file.
    // None if no chunks are being loaded at the moment
    world_change: Option<WorldChange>,
}

fn get_difference_1d(region: i32, chunk: usize, new_region: i32, new_chunk: usize) -> i32 {
    if region == new_region {
        if chunk == new_chunk {
            return 0;
        }

        if chunk > new_chunk {
            return -1;
        }

        return 1;
    }

    if region > new_region {
        return -1;
    }

    return 1;
}

fn get_difference(original: &ChunkPosition, different: &ChunkPosition) -> (i32, i32) {
    let diff_x = get_difference_1d(
        original.region_x,
        original.chunk_x,
        different.region_x,
        different.chunk_x,
    );
    let diff_z = get_difference_1d(
        original.region_z,
        original.chunk_z,
        different.region_z,
        different.chunk_z,
    );

    (diff_x, diff_z)
}

const OFFSET_FROM_CENTER: usize = config::WORLD_SIZE / 2;

impl World {
    pub fn new(position: Position) -> Self {
        let center_chunk_position = get_minecraft_chunk_position(position);

        // Get position of chunk that corresponds to 0,0 in the world grid
        let base_chunk_position = center_chunk_position
            .offset(-(OFFSET_FROM_CENTER as i32), -(OFFSET_FROM_CENTER as i32));

        let (tx, rx) = mpsc::channel();

        World {
            chunks: array_init(|index| {
                let x = index % config::WORLD_SIZE;
                let z = index / config::WORLD_SIZE;
                let chunk_position = base_chunk_position.offset(x as i32, z as i32);

                let mut chunk = minecraft::get_chunk(chunk_position);
                chunk.build_surface();

                Arc::new(chunk)
            }),
            chunk_meshes: array_init(|_| Lazy::new()),
            center: center_chunk_position,
            mesh_sender: tx,
            mesh_receiver: rx,
            mesh_builders: Vec::new(),
            meshes_being_built: HashSet::new(),
            world_change: None,
        }
    }

    // Note: this allocates a bunch of *unnecessary* vectors
    // but I'm not sure if there is another way
    pub fn get_surface_block_data(&self) -> Vec<BlockData> {
        self.chunks
            .iter()
            .flat_map(|chunk| chunk.surface_blocks.clone())
            .collect_vec()
    }

    pub fn get_rigid_blocks_data(&self) -> Vec<BlockData> {
        self.chunks
            .iter()
            .flat_map(|chunk| chunk.get_rigid_block_data().into_iter())
            .collect()
    }

    // Do not filter the blocks in any way - for debug purposes
    #[allow(unused)]
    pub fn get_block_data(&self) -> Vec<BlockData> {
        let mut blocks = Vec::<BlockData>::new();

        for chunk in &self.chunks {
            let mut chunk_blocks = chunk.get_block_data();
            blocks.append(&mut chunk_blocks);
        }

        blocks
    }

    fn apply_chunk_change(&mut self, change: ChunkChange) {
        let chunk_index = change.0;

        match change.1 {
            ChunkSource::Direct(chunk) => {
                self.chunks[chunk_index] = Arc::new(chunk);
                self.chunk_meshes[chunk_index] = Lazy::new();
            }
            ChunkSource::Reference(new_chunk_index) => {
                self.chunks.swap(chunk_index, new_chunk_index);
                self.chunk_meshes.swap(chunk_index, new_chunk_index);
            }
        }
    }

    fn integrate_world_change(&mut self) -> bool {
        let Some(world_change) = self.world_change.take() else {
            return false;
        };

        let new_center = world_change.0;
        let builder = world_change.1;

        match builder.join() {
            Ok(chunk_changes) => {
                // The changes have to be applied in a specific order
                for change in chunk_changes {
                    self.apply_chunk_change(change);
                }
                self.center = new_center;

                return true;
            }
            Err(panic_message) => {
                println!("Chunk builder thread panicked! Recentering to {new_center:?} was aborted. --\n{panic_message:?}");

                return false;
            }
        }
    }

    // This method does not do the actual updating.
    // Instead, it will manage the worker thread that does it.
    // Returns true if a new part of the world was loaded
    pub fn update_chunk_data(
        &mut self,
        new_position: Position,
        options: PolygonizationOptions,
    ) -> bool {
        // Only 1 update can be running at any time
        if let Some(world_change) = &self.world_change {
            let builder = &world_change.1;
            let in_progress = !builder.is_finished();
            if in_progress {
                return false;
            }
        }

        // There is no update running, or it has finished, itegrate the changes, if any.
        let world_data_updated = self.integrate_world_change();
        if world_data_updated {
            self.dispatch_mesh_builder(options);
        }

        // Check whether we need to update and dispatch the update task.
        let center_chunk_position = get_minecraft_chunk_position(new_position);
        let recenter = self.center != center_chunk_position;
        if recenter {
            let chunks = self.get_chunks();
            let direction_of_change = get_difference(&self.center, &center_chunk_position);

            let handle = thread::spawn(move || {
                time_it!(
                    "Offset chunks",
                    let x = World::offset_chunks(chunks, direction_of_change);
                );

                x
            });
            self.world_change = Some(WorldChange(center_chunk_position, handle));
        }

        world_data_updated
    }

    // Returns whether any meshes were updated.
    //
    // We only return true in case a whole batch was finished,
    // even if we have some meshes queued up.
    pub fn update_smooth_mesh(&mut self) -> bool {
        self.integrate_built_meshes();
        let any_finished = self.join_finished_workers();

        any_finished
    }

    fn integrate_built_meshes(&mut self) {
        let mut recv_result = self.mesh_receiver.try_recv();
        while let Ok(data) = recv_result {
            let mesh = data.0;
            let chunk_position = data.1;

            self.meshes_being_built.remove(&chunk_position);

            let target_index = self.chunks.iter().enumerate().find_map(|(index, chunk)| {
                if chunk.position == chunk_position {
                    Some(index)
                } else {
                    None
                }
            });

            if let Some(mesh_index) = target_index {
                assert!(
                    self.chunk_meshes[mesh_index].get().is_none(),
                    "The mesh for {chunk_position:?} was already built!"
                );
                self.chunk_meshes[mesh_index].get_or_create(|| mesh);
            } else {
                println!(
                    "Received mesh for chunk {:?}, but that chunk is not loaded!",
                    chunk_position
                );
            }

            recv_result = self.mesh_receiver.try_recv();
        }
    }

    fn join_finished_workers(&mut self) -> bool {
        let finished_threads_indices = self
            .mesh_builders
            .iter()
            .enumerate()
            .filter_map(|(index, handle)| {
                if handle.is_finished() {
                    Some(index)
                } else {
                    None
                }
            })
            .sorted()
            .rev()
            .collect_vec();

        let any_finished = !finished_threads_indices.is_empty();

        // The threads are removed from largest index to smallest
        // This way the indices stay valid since swap_remove always replaced the element with the
        // last element of the vector
        for thread_index in finished_threads_indices {
            let handle = self.mesh_builders.swap_remove(thread_index);

            let join_result = handle.join();
            match join_result {
                Ok(send_errors) if !send_errors.is_empty() => {
                    let error_message = send_errors
                        .into_iter()
                        .map(|err| {
                            let payload = &err.0;
                            let chunk_position = payload.1;

                            // Make sure to clear the hash set and not leak memory
                            // We have the built mesh here, why not integrate it even if it
                            // failed? ... Maybe we don't need the channel after all.
                            self.meshes_being_built.remove(&chunk_position);

                            err.to_string()
                        })
                        .join(", ");

                    println!("The following errors occured when trying to send to the channel:\n {error_message}");
                }
                Err(panic_message) => println!("Worker thread panicked! - {panic_message:?}"),
                _ => (), /* println!("Successfully joined worker thread.") */
            };
        }

        return any_finished;
    }

    fn offset_chunks(chunks: WorldChunks, offset: (i32, i32)) -> Vec<ChunkChange> {
        let (direction_x, direction_z) = offset;
        let reverse_x = direction_x < 0;
        let reverse_z = direction_z < 0;

        let index_builder = |reverse: bool| -> [usize; WORLD_SIZE] {
            array_init(|index| match reverse {
                true => WORLD_SIZE - 1 - index,
                false => index,
            })
        };

        let swappable_region_size = |dir: i32| -> usize {
            let loading_in_direction = dir != 0;

            if loading_in_direction {
                WORLD_SIZE - 1
            } else {
                WORLD_SIZE
            }
        };

        let x_iter = index_builder(reverse_x)
            .into_iter()
            .take(swappable_region_size(direction_x));
        let z_iter = index_builder(reverse_z)
            .into_iter()
            .take(swappable_region_size(direction_z));

        let swappable_chunks_iterator = x_iter.cartesian_product(z_iter);

        let chunks_swaps = swappable_chunks_iterator.map(|(x, z)| {
            let current_chunk_index = World::chunk_index(x, z);

            let next_x = (x as i32 + direction_x) as usize;
            let next_z = (z as i32 + direction_z) as usize;
            let next_chunk_index = World::chunk_index(next_x, next_z);

            let swap_chunks = ChunkChange(
                current_chunk_index,
                ChunkSource::Reference(next_chunk_index),
            );

            swap_chunks
        });

        // Iter all indices that couldn't be swapped.
        // These are the indices on the edges that correspond to the offset direction
        let indices_of_chunks_to_load = {
            let edge_coord = |reverse: bool| match reverse {
                true => 0,
                false => WORLD_SIZE - 1,
            };
            let x_edge_coord = edge_coord(reverse_x);
            let z_edge_coord = edge_coord(reverse_z);

            let x_edge_indices = (0..WORLD_SIZE).map(|z| (x_edge_coord, z));
            let z_edge_indices = (0..WORLD_SIZE).map(|x| (x, z_edge_coord));

            if direction_x == 0 {
                z_edge_indices.collect_vec()
            } else if direction_z == 0 {
                x_edge_indices.collect_vec()
            } else {
                x_edge_indices.chain(z_edge_indices).unique().collect_vec()
            }
        };

        let chunk_loads = indices_of_chunks_to_load.into_iter().map(|(x, z)| {
            let current_chunk_index = World::chunk_index(x, z);

            let original_position = &chunks[current_chunk_index].position;
            let position_to_load = original_position.offset(direction_x, direction_z);

            time_it!("Chunk LOAD",
            let mut chunk = minecraft::get_chunk(position_to_load);
            chunk.build_surface();
            );

            let chunk_load = ChunkChange(current_chunk_index, ChunkSource::Direct(chunk));

            chunk_load
        });

        chunks_swaps.chain(chunk_loads).collect_vec()
    }

    fn chunk_index(x: usize, z: usize) -> usize {
        z * config::WORLD_SIZE + x
    }

    pub fn get_block(&self, position: Position) -> BlockType {
        let chunk_position = get_minecraft_chunk_position(position);
        let chunk = self
            .chunks
            .iter()
            .find(|chunk| chunk.position == chunk_position);

        let Some(chunk) = chunk else {
            return BlockType::Air;
        };

        let (block_x, block_z) = Chunk::get_block_coords(position.x, position.z);
        chunk.get_block(block_x, position.y.floor() as isize, block_z)
    }

    // TODO: this can be const and return fixed sized array that depends on WORLD_SIZe
    fn inner_chunk_indices() -> Vec<usize> {
        // To evaluate the sdf at a point, we need data in a radius around that point.
        // For the chunks that are on the edges of the (loaded) world we are missing data,
        // resulting in artifacts when stitching the chunk meshes together.
        //
        // For now the simple solution is just to polygonize only the chunks that have all
        // neighboring chunks loaded.
        let chunk_indices = (1..WORLD_SIZE - 1)
            .cartesian_product(1..WORLD_SIZE - 1)
            .map(|(x, z)| World::chunk_index(x, z))
            .collect::<Vec<usize>>();

        chunk_indices
    }

    pub fn get_chunks(&self) -> WorldChunks {
        clone_world(&self.chunks)
    }

    pub fn get_smooth_mesh(&self) -> Mesh {
        let chunk_meshes = World::inner_chunk_indices()
            .into_iter()
            .filter_map(|index| {
                let chunk_mesh = &self.chunk_meshes[index];
                chunk_mesh.get()
            });

        Mesh::copy_merge(chunk_meshes)
    }

    pub fn dispatch_mesh_builder(&mut self, options: PolygonizationOptions) {
        let chunks = self.get_chunks();

        let chunks_without_mesh = World::inner_chunk_indices()
            .into_iter()
            .filter(|index| {
                let chunk_position = self.chunks[*index].position;
                let chunk_mesh = &self.chunk_meshes[*index];

                chunk_mesh.get().is_none() && !self.meshes_being_built.contains(&chunk_position)
            })
            .map(|index| (index, self.mesh_sender.clone()))
            .collect_vec();

        // Avoid spawning the worker thread when not needed
        if chunks_without_mesh.is_empty() {
            return;
        }

        println!(
            "[INFO] Starting of {} meshes with cell resolution {}.",
            chunks_without_mesh.len(),
            options.marching_cubes_cell_size
        );

        let positions_to_build = chunks_without_mesh
            .iter()
            .map(|(index, _)| self.chunks[*index].position);
        self.meshes_being_built.extend(positions_to_build);

        let work_handle = thread::spawn(move || {
            let n = chunks_without_mesh.len();

            time_it!("Building meshes of smooth surfaces",
                let send_errors = chunks_without_mesh
                    //.into_iter() // serial implementation
                    .into_par_iter() // parallel implementation
                    .filter_map(|(index, tx)| {
                        let chunk_mesh = polygonize_chunk(&chunks, index, options);
                        let chunk_position = chunks[index].position;
                        let payload = BoundMesh(chunk_mesh, chunk_position);

                        if let Err(send_error) = tx.send(payload) {
                            Some(send_error)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<SendError<BoundMesh>>>();
            );
            println!("[INFO] Built {} smooth chunk meshes.", n);

            send_errors
        });

        self.mesh_builders.push(work_handle);
    }

    pub fn rebuild_all_meshes(&mut self, options: PolygonizationOptions) {
        for i in 0..CHUNKS_IN_WORLD {
            self.chunk_meshes[i] = Lazy::new();
        }

        self.dispatch_mesh_builder(options);
    }
}
