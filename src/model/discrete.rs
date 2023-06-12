use array_init::array_init;
use itertools;
use lazy_init::Lazy;
use std::cmp::max;
use std::cmp::min;

use crate::config;
use crate::config::WORLD_SIZE;
use crate::get_minecraft_chunk_position;
use crate::infrastructure::texture::MaterialBlend;
use crate::minecraft;

use super::chunk::{BlockData, Chunk, ChunkPosition};
use super::common::get_pallette_texture_coords;
use super::common::is_visible_block;
use super::common::BlockType;
use super::implicit::Kernel;
use super::Coord;
use super::Position;
use super::Real;
use super::implicit::sdf_unit_cube_exact;
use super::polygonize::Mesh;
use super::polygonize::Rectangle3D;

const CHUNKS_IN_WORLD: usize = WORLD_SIZE * WORLD_SIZE;

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
    chunks: [Chunk; CHUNKS_IN_WORLD],

    chunk_meshes: [Lazy<Mesh>; CHUNKS_IN_WORLD],

    // TODO: make private and get the world support to polygonize another way
    // Position of the center chunk in the world
    pub center: ChunkPosition,
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

        World {
            chunks: array_init(|index| {
                let x = index % config::WORLD_SIZE;
                let z = index / config::WORLD_SIZE;
                let chunk_position = base_chunk_position.offset(x as i32, z as i32);

                minecraft::get_chunk(chunk_position)
            }),
            chunk_meshes: array_init(|_| Lazy::new()),
            center: center_chunk_position,
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
    pub fn update(&mut self, new_position: Position) -> bool {
        let new_center_chunk = get_minecraft_chunk_position(new_position);
        if self.center == new_center_chunk {
            return false;
        }

        // Get the direction of change
        let (direction_x, direction_z) = get_difference(&self.center, &new_center_chunk);
        self.center = new_center_chunk;

        // Update the world matrix by either shifting chunks based on the direction, or loading
        // needed chunks
        let reverse_x = direction_x < 0;
        let reverse_z = direction_z < 0;

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

                let new_chunk = minecraft::get_chunk(position_to_load);

                self.chunks[current_chunk_index] = new_chunk;
                self.chunk_meshes[current_chunk_index] = Lazy::new();
            }
        }

        return true;
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

    pub fn sample_volume(&self, kernel: Kernel) -> Real {
        let kernel_box = kernel.get_bounding_rectangle();
        let y_low = kernel.y_low();
        let y_high = kernel.y_high();

        self.chunks.iter().fold(0.0, |acc, chunk| {
            let chunk_box = chunk.get_bounding_rectangle();
            let Some(intersection) = chunk_box.intersect(kernel_box) else {
                return acc;
            };

            let offset = chunk.position.get_global_position().map(|coord| -coord);
            let intersection_local = intersection.offset_origin(offset);
            let chunk_volume =
                chunk.get_chunk_intersection_volume(intersection_local, y_low, y_high);

            acc + chunk_volume
        })
    }

    pub fn distance_to_rigid_blocks(&self, point: Position) -> Option<Real> {
        let kernel = Kernel::new(point, 0.5);
        let kernel_box = kernel.get_bounding_rectangle();

        let intersected_chunks = self.chunks.iter().filter(|chunk| {
            chunk
                .get_bounding_rectangle()
                .intersect(kernel_box)
                .is_some()
        });

        let closest_rigid_block_per_chunk = intersected_chunks
            .map(|chunk| chunk.get_closest_rigid_block(point))
            .filter_map(|rigid_block_option| rigid_block_option);

        let Some(closest_rigid_block) =
            closest_rigid_block_per_chunk.fold(None, |min_dist, dist| match min_dist {
                None => Some(dist),
                Some(val) => {
                    if dist.1 < val.1 {
                        Some(dist)
                    } else {
                        min_dist
                    }
                }
            }) 
        else {
            return None;
        };

        let block_position = closest_rigid_block.0.position;
        let block_local_point = point.zip(block_position, |k, b| k - b);


        Some(sdf_unit_cube_exact(block_local_point))
    }

    pub fn sample_materials(&self, kernel: Kernel) -> MaterialBlend {
        let kernel_box = kernel.get_bounding_rectangle();
        let y_low = kernel.y_low();
        let y_high = kernel.y_high();

        self.chunks
            .iter()
            .fold(MaterialBlend::new(), |mut blend, chunk| {
                let chunk_box = chunk.get_bounding_rectangle();
                let Some(intersection) = chunk_box.intersect(kernel_box) else {
                    return blend;
                };

                let offset = chunk.position.get_global_position().map(|coord| -coord);
                let intersection_local = intersection.offset_origin(offset);
                let chunk_volume = chunk.get_material_blend(intersection_local, y_low, y_high);

                blend.merge(chunk_volume);
                blend
            })
    }

    fn polygonize_chunk(&self, chunk: &Chunk) -> Mesh {
        let support_xz = chunk.position.get_global_position();

        let support_low_y = 40.0; // TODO: use MIN_Y
        let support_y_size = 40.0; // TODO: use full chunk height

        let support = Rectangle3D {
            position: Position::new(support_xz.x, support_low_y, support_xz.y),
            width: minecraft::BLOCKS_IN_CHUNK as Real,
            depth: minecraft::BLOCKS_IN_CHUNK as Real,
            height: support_y_size,
        };

        let density_func = |p| super::implicit::evaluate_density_rigid(self, p);
        let material_func = |p| super::implicit::sample_materials(self, p);

        super::polygonize::polygonize(support, density_func, material_func)
    }

    // 1. calculate support for each chunk
    // 2. polygonize each chunk separately
    // 3. merge the meshes of each chunk into one big mesh,
    //    that is the mesh we return
    pub fn polygonize(&self) -> Mesh {
        let mut world_mesh = Mesh::empty();

        // To evaluate the sdf at a point, we need data in a radius around that point.
        // For the chunks that are on the edges of the (loaded) world we are missing data,
        // resulting in artifacts when stitching the chunk meshes together.
        //
        // For now the simple solution is just to polygonize only the chunks that have all
        // neighboring chunks loaded.
        for x in 1..WORLD_SIZE - 1 {
            for z in 1..WORLD_SIZE - 1 {
                let chunk_index = World::chunk_index(x, z);

                let chunk = &self.chunks[chunk_index];

                let mesh_creator = || self.polygonize_chunk(chunk);
                let chunk_mesh = self.chunk_meshes[chunk_index].get_or_create(mesh_creator);

                chunk_mesh.copy_into(&mut world_mesh);
            }
        }

        world_mesh
    }
}
