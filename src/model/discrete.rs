use array_init::array_init;
use fastanvil::RegionFileLoader;
use itertools;
use itertools::Itertools;
use lazy_init::Lazy;
use rayon::prelude::*;
use std::cmp::max;
use std::cmp::min;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SendError;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Instant;

use crate::config;
use crate::config::WORLD_SIZE;
use crate::get_minecraft_chunk_position;
use crate::minecraft;
use crate::model::implicit::smooth::polygonize_chunk;

use super::chunk::{BlockData, Chunk, ChunkPosition};
use super::common::get_pallette_texture_coords;
use super::common::is_visible_block;
use super::common::BlockType;
use super::polygonize::Mesh;
use super::Coord;
use super::Position;

const CHUNKS_IN_WORLD: usize = WORLD_SIZE * WORLD_SIZE;

pub type WorldChunks = [Arc<Chunk>; CHUNKS_IN_WORLD];

fn clone_world(chunks: &WorldChunks) -> WorldChunks {
    let clone = array_init(|index| chunks[index].clone());
    clone
}

// A mesh of a chunk located at *ChunkPosition*
struct BoundMesh(Mesh, ChunkPosition);

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
    chunk_meshes: [Lazy<Mesh>; CHUNKS_IN_WORLD],

    // Position of the center chunk in the world
    center: ChunkPosition,

    mesh_sender: Sender<BoundMesh>,
    mesh_receiver: Receiver<BoundMesh>,
    working_threads: Vec<JoinHandle<Vec<SendError<BoundMesh>>>>,

    region_loader: RegionFileLoader,
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

fn clamp_chunk_index(i: i32) -> Option<usize> {
    if i >= 0 && (i as usize) < config::WORLD_SIZE {
        return Some(i as usize);
    }

    None
}

fn get_iterator(
    from: usize,
    to: usize,
    reverse: bool,
) -> itertools::Either<impl Iterator<Item = usize>, impl Iterator<Item = usize>> {
    if reverse {
        itertools::Either::Left((from..to).rev())
    } else {
        itertools::Either::Right(from..to)
    }
}

fn create_block_data(position: Position, material: BlockType) -> BlockData {
    BlockData {
        offset: [position.x as f32, position.y as f32, position.z as f32],
        pallette_offset: get_pallette_texture_coords(material),
    }
}

const BLOCK_SIZE: Coord = 1.0;
const OFFSET_FROM_CENTER: usize = config::WORLD_SIZE / 2;

impl World {
    pub fn new(position: Position) -> Self {
        let center_chunk_position = get_minecraft_chunk_position(position);

        // Get position of chunk that corresponds to 0,0 in the world grid
        let base_chunk_position = center_chunk_position
            .offset(-(OFFSET_FROM_CENTER as i32), -(OFFSET_FROM_CENTER as i32));

        let (tx, rx) = mpsc::channel();

        let loader = RegionFileLoader::new(config::WORLD_FOLDER.into());

        World {
            chunks: array_init(|index| {
                let x = index % config::WORLD_SIZE;
                let z = index / config::WORLD_SIZE;
                let chunk_position = base_chunk_position.offset(x as i32, z as i32);

                Arc::new(minecraft::get_chunk(&loader, chunk_position))
            }),
            chunk_meshes: array_init(|_| Lazy::new()),
            center: center_chunk_position,
            mesh_sender: tx,
            mesh_receiver: rx,
            working_threads: Vec::new(),
            region_loader: loader,
        }
    }

    // A block is only visible if there is at least 1 air block
    // in its neighborhood
    fn is_position_visible(&self, position: Position) -> bool {
        let left = Position::new(position.x - BLOCK_SIZE, position.y, position.z);
        let right = Position::new(position.x + BLOCK_SIZE, position.y, position.z);
        let down = Position::new(position.x, position.y - BLOCK_SIZE, position.z);
        let up = Position::new(position.x, position.y + BLOCK_SIZE, position.z);
        let forward = Position::new(position.x, position.y, position.z - BLOCK_SIZE);
        let back = Position::new(position.x, position.y, position.z + BLOCK_SIZE);

        let neighbors = [left, right, up, down, forward, back];
        return neighbors
            .into_iter()
            .any(|pos| !is_visible_block(self.get_block(pos))); // @Speed
    }

    pub fn get_surface_block_data(&self, y_low: isize, y_high: isize) -> Vec<BlockData> {
        self.chunks
            .iter()
            .flat_map(|chunk| {
                let chunk_offset = chunk.position.get_global_position();
                // println!("pos {:?} offset {:?}", chunk.position, chunk_offset);

                chunk
                    .enumerate_blocks(y_low, y_high)
                    .map(move |(relative_position, material)| {
                        let position = Position::new(
                            relative_position.x + chunk_offset.x as Coord,
                            relative_position.y,
                            relative_position.z + chunk_offset.y as Coord,
                        );

                        (position, material)
                    })
                    .filter(|(position, _)| self.is_position_visible(*position))
                    .map(move |(position, material)| create_block_data(position, material))
            })
            .collect()
    }

    pub fn get_rigid_blocks_data(&self) -> Vec<BlockData> {
        self.chunks
            .iter()
            .flat_map(|chunk| chunk.get_rigid_block_data().into_iter())
            .collect()
    }

    pub fn get_block_data(&self) -> Vec<BlockData> {
        let mut blocks = Vec::<BlockData>::new();

        for chunk in &self.chunks {
            let mut chunk_blocks = chunk.get_block_data();
            blocks.append(&mut chunk_blocks);
        }

        blocks
    }

    // Returns true if a new part of the world was loaded
    pub fn update_chunk_data(&mut self, new_position: Position) -> bool {
        let center_chunk_position = get_minecraft_chunk_position(new_position);
        let recenter = self.center != center_chunk_position;
        if recenter {
            self.center_loaded_chunks(center_chunk_position);
            self.dispatch_mesh_builder();
        }

        recenter
    }

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

            //println!("received mesh for chunk at {chunk_position:?}");

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
            .working_threads
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
            let handle = self.working_threads.swap_remove(thread_index);

            let join_result = handle.join();
            match join_result {
                Ok(send_errors) if !send_errors.is_empty() => {
                    let error_message = send_errors
                        .into_iter()
                        .map(|err| err.to_string())
                        .join(", ");

                    println!("The following errors occured when trying to send to the channel:\n {error_message}");
                }
                Err(panic_message) => println!("Worker thread panicked! - {panic_message:?}"),
                _ => println!("Successfully joined worker thread."),
            };
        }

        return any_finished;
    }

    fn center_loaded_chunks(&mut self, center_chunk_position: ChunkPosition) {
        // Get the direction of change
        let (direction_x, direction_z) = get_difference(&self.center, &center_chunk_position);
        self.center = center_chunk_position;

        // Update the world matrix by either shifting chunks based on the direction, or loading
        // needed chunks
        let reverse_x = direction_x < 0;
        let reverse_z = direction_z < 0;

        let mut chunks_loaded = 0;
        let now = Instant::now();

        // TODO: comment what does this do
        for z in get_iterator(0, WORLD_SIZE, reverse_z) {
            for x in get_iterator(0, WORLD_SIZE, reverse_x) {
                let next_x = x as i32 + direction_x;
                let next_z = z as i32 + direction_z;

                let current_chunk_index = World::chunk_index(x, z);
                if let Some(next_x) = clamp_chunk_index(next_x) {
                    if let Some(next_z) = clamp_chunk_index(next_z) {
                        let next_index = World::chunk_index(next_x, next_z);

                        self.chunks.swap(current_chunk_index, next_index);
                        self.chunk_meshes.swap(current_chunk_index, next_index);
                        continue;
                    }
                }

                let original_x =
                    min(max(x as i32 - direction_x, 0), WORLD_SIZE as i32 - 1) as usize;
                let original_z =
                    min(max(z as i32 - direction_z, 0), WORLD_SIZE as i32 - 1) as usize;

                let original_index = World::chunk_index(original_x, original_z);
                let original_position = &self.chunks[original_index].position;
                let position_to_load = original_position.offset(direction_x, direction_z);

                let now = Instant::now();
                let new_chunk =
                    Arc::new(minecraft::get_chunk(&self.region_loader, position_to_load));

                let elapsed = now.elapsed();
                println!("-------------- getting new chunk took {elapsed:.2?}");

                self.chunks[current_chunk_index] = new_chunk;
                self.chunk_meshes[current_chunk_index] = Lazy::new();
                chunks_loaded += 1;
            }
        }

        let elapsed = now.elapsed();
        println!(
            "Update world took {:.2?} ({} new chunks loaded)",
            elapsed, chunks_loaded
        );
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

    //    pub fn sample_volume(&self, kernel: Kernel) -> Real {
    //        let kernel_box = kernel.get_bounding_rectangle();
    //        let y_low = kernel.y_low();
    //        let y_high = kernel.y_high();
    //
    //        self.chunks.iter().fold(0.0, |acc, chunk| {
    //            let chunk_box = chunk.get_bounding_rectangle();
    //            let Some(intersection) = chunk_box.intersect(kernel_box) else {
    //                return acc;
    //            };
    //
    //            let offset = chunk.position.get_global_position().map(|coord| -coord);
    //            let intersection_local = intersection.offset_origin(offset);
    //            let chunk_volume =
    //                chunk.get_chunk_intersection_volume(intersection_local, y_low, y_high);
    //
    //            acc + chunk_volume
    //        })
    //    }

    //    pub fn distance_to_rigid_blocks(&self, point: Position) -> Option<Real> {
    //        let kernel = Kernel::new(point, 0.5);
    //        let kernel_box = kernel.get_bounding_rectangle();
    //
    //        let intersected_chunks = self.chunks.iter().filter(|chunk| {
    //            chunk
    //                .get_bounding_rectangle()
    //                .intersect(kernel_box)
    //                .is_some()
    //        });
    //
    //        let closest_rigid_block_per_chunk = intersected_chunks
    //            .map(|chunk| chunk.get_closest_rigid_block(point))
    //            .filter_map(|rigid_block_option| rigid_block_option);
    //
    //        let Some(closest_rigid_block) =
    //            closest_rigid_block_per_chunk.fold(None, |min_dist, dist| match min_dist {
    //                None => Some(dist),
    //                Some(val) => {
    //                    if dist.1 < val.1 {
    //                        Some(dist)
    //                    } else {
    //                        min_dist
    //                    }
    //                }
    //            })
    //        else {
    //            return None;
    //        };
    //
    //        let block_position = closest_rigid_block.0.position;
    //        let block_local_point = point.zip(block_position, |k, b| k - b);
    //
    //
    //        Some(sdf_unit_cube_exact(block_local_point))
    //    }

    //    pub fn sample_materials(&self, kernel: Kernel) -> MaterialBlend {
    //        let kernel_box = kernel.get_bounding_rectangle();
    //        let y_low = kernel.y_low();
    //        let y_high = kernel.y_high();
    //
    //        self.chunks
    //            .iter()
    //            .fold(MaterialBlend::new(), |mut blend, chunk| {
    //                let chunk_box = chunk.get_bounding_rectangle();
    //                let Some(intersection) = chunk_box.intersect(kernel_box) else {
    //                    return blend;
    //                };
    //
    //                let offset = chunk.position.get_global_position().map(|coord| -coord);
    //                let intersection_local = intersection.offset_origin(offset);
    //                let chunk_volume = chunk.get_material_blend(intersection_local, y_low, y_high);
    //
    //                blend.merge(chunk_volume);
    //                blend
    //            })
    //    }

    //    fn polygonize_chunk(&self, chunk: &Chunk) -> Mesh {
    //        let support_xz = chunk.position.get_global_position();
    //
    //        let support_low_y = 40.0; // TODO: use MIN_Y
    //        let support_y_size = 40.0; // TODO: use full chunk height
    //
    //        let support = Rectangle3D {
    //            position: Position::new(support_xz.x, support_low_y, support_xz.y),
    //            width: minecraft::BLOCKS_IN_CHUNK as Real,
    //            depth: minecraft::BLOCKS_IN_CHUNK as Real,
    //            height: support_y_size,
    //        };
    //
    //        let density_func = |p| super::implicit::evaluate_density_rigid(self, p);
    //        let material_func = |p| super::implicit::sample_materials(self, p);
    //
    //        super::polygonize::polygonize(support, density_func, material_func)
    //    }

    // 1. calculate support for each chunk
    // 2. polygonize each chunk separately
    // 3. merge the meshes of each chunk into one big mesh,
    //    that is the mesh we return
    //    pub fn polygonize(&self) -> (Mesh, usize) {
    //        // To evaluate the sdf at a point, we need data in a radius around that point.
    //        // For the chunks that are on the edges of the (loaded) world we are missing data,
    //        // resulting in artifacts when stitching the chunk meshes together.
    //        //
    //        // For now the simple solution is just to polygonize only the chunks that have all
    //        // neighboring chunks loaded.
    //        let chunk_indices = (1..WORLD_SIZE - 1)
    //            .cartesian_product(1..WORLD_SIZE - 1)
    //            .map(|(x, z)| World::chunk_index(x, z))
    //            .collect::<Vec<usize>>();
    //
    //        let chunks_without_mesh = chunk_indices
    //            .iter()
    //            .filter(|&index| self.chunk_meshes[*index].get().is_none())
    //            .count();
    //
    //        // Create the meshes in parallel
    //        let chunk_meshes = chunk_indices
    //            .into_par_iter()
    //            .map(|chunk_index| {
    //                let chunk = &self.chunks[chunk_index];
    //                let mesh_creator = || Mesh::empty(); //self.polygonize_chunk(chunk);
    //
    //                let chunk_mesh = self.chunk_meshes[chunk_index].get_or_create(mesh_creator);
    //                chunk_mesh
    //            })
    //            .collect::<Vec<&Mesh>>();
    //
    //        let world_mesh = Mesh::merge(chunk_meshes.into_iter());
    //
    //        //        // Serial implementation
    //        //        let mut world_mesh = Mesh::empty();
    //        //        for x in 1..WORLD_SIZE - 1 {
    //        //            for z in 1..WORLD_SIZE - 1 {
    //        //                let chunk_index = World::chunk_index(x, z);
    //        //
    //        //                let chunk = &self.chunks[chunk_index];
    //        //
    //        //                let mesh_creator = || self.polygonize_chunk(chunk);
    //        //                let chunk_mesh = self.chunk_meshes[chunk_index].get_or_create(mesh_creator);
    //        //
    //        //                chunk_mesh.copy_into(&mut world_mesh);
    //        //            }
    //        //        }
    //
    //        (world_mesh, chunks_without_mesh)
    //    }

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

        Mesh::merge(chunk_meshes)
    }

    pub fn dispatch_mesh_builder(&mut self) {
        let chunks = self.get_chunks();

        let chunks_without_mesh = World::inner_chunk_indices()
            .into_iter()
            .filter(|index| {
                let chunk_mesh = &self.chunk_meshes[*index];
                chunk_mesh.get().is_none()
            })
            .map(|index| (index, self.mesh_sender.clone()))
            .collect_vec();

        // Avoid spawning the worker thread when not needed
        if chunks_without_mesh.is_empty() {
            return;
        }

        let work_handle = thread::spawn(move || {
            let work_start = Instant::now();

            let n = chunks_without_mesh.len();
            let send_errors = chunks_without_mesh
                .into_par_iter()
                .filter_map(|(index, tx)| {
                    let chunk_mesh = polygonize_chunk(&chunks, index);
                    let chunk_position = chunks[index].position;
                    let payload = BoundMesh(chunk_mesh, chunk_position);

                    if let Err(send_error) = tx.send(payload) {
                        Some(send_error)
                    } else {
                        None
                    }
                })
                .collect::<Vec<SendError<BoundMesh>>>();

            let work_time = work_start.elapsed();
            println!("Building mesh for {n} chunks took {work_time:.2?}.");

            send_errors
        });

        self.working_threads.push(work_handle);

        // spawn a thread that runs the polygonization code in parallel
        // save a handle to the thread
        // the thread should return the computed meshes after it is done.
        // the meshes will be saved by some other method ??
        // this method does not block
        //
        // make chunks into Arc<Chunk> so they can be sent across threads
        // create a new array with cloned arcs of chunks for the background thread
        // the indices will be computed in the current thread and passed as precomputed data
        // the thread will return computed meshes along with indices of the meshes
        // get_smooth_mesh will then check if the thread is finished and if so, populate the lazy
        // meshes with newly computed values before doing its thing
    }
}
