use cgmath::{EuclideanSpace, InnerSpace, Point3};
use glium::implement_vertex;

use crate::model::{implicit, Position, Real};

// The jump in quality between 1.0 and 0.9 is insane!
const CELL_SIZE: Real = 1.0;
const SURFACE_LEVEL: Real = 0.0;

pub struct Mesh {
    // Vertices of the mesh
    pub vertices: Vec<MeshVertex>,

    // Indices of vertices forming triangles
    // len(indices) = 3 * len(vertices)
    pub indices: Vec<u32>,
}

// For each cell evaluate this many edge intersections,
// the rest are evaluated as part of the <n> intersections evaluated for some other cell.
//
// This way each grid edge will only get evaluated once and it can be done in parallel
const INTERSECTION_STRIDE: usize = 3;

// Which edges each cell will evaluate
// The values correspont to edge indices within the cube
const EDGE_INDICES: [u16; INTERSECTION_STRIDE] = [
    0, /*back edge*/
    3, /*right edge*/
    8, /*up edge*/
];

// Grid has n points
// this vector is 3*n long
// for point i, this vector contains information about the back,right and up instersection
// at index 3*i, 3*i + 1, 3*i + 2 respectively
type Intersection = Option<Position>;
type IntersectionContainer = Vec<Intersection>;

// Has the same length as IntersectionContainer
// each element maps Intersection with the matching index to the vertex index in the resulting
// vertex buffer
// Option variants should match for each index
//
// The mapping is decoupled from the Intersection to allow parallelization
type IntersectionVertexMap = Vec<Option<u32>>;

// Driver function, returns vertex+index buffer and dispatches work
pub fn polygonize(support: Rectangle3D, density_func: impl Fn(Position) -> Real) -> Mesh {
    let grid = Grid::new(support, &density_func);
    let intersections = find_intersections(&grid);
    //let index = grid.get_index_for(GridPosition::new(1, 2, 0));
    //    println!("intersections: {}", intersections.len());
    //    println!(
    //        "1, 2, 0 is at index {} edges {:?} {:?} {:?}",
    //        index,
    //        intersections[index],
    //        intersections[index + 1],
    //        intersections[index + 2]
    //    );

    let (vertices, vertex_mapping) = build_mesh_vertices(&intersections, &density_func);
    //    println!(
    //        "mesh vertices: {}, map length: {}, map some: {}, matches: {}",
    //        vertices.len(),
    //        vertex_mapping.len(),
    //        vertex_mapping.iter().filter(|x| x.is_some()).count(),
    //        vertex_mapping
    //            .iter()
    //            .enumerate()
    //            .all(|(i, x)| x.is_some() == intersections[i].is_some()),
    //    );

    let indices = assemble_triangles(&grid, &vertex_mapping);

    Mesh { vertices, indices }
}

// Return a collection of mesh vertices + a mapping of intersections to the new vertices.
// The vertices are in the same order they came in
//
// The vertices collection only contains actual vertices so its shorter than the intersections
// collection which contains also None values.
// For this reason a mapping of Intersection -> MeshVertex is required
fn build_mesh_vertices(
    intersections: &IntersectionContainer,
    density_func: &impl Fn(Position) -> Real,
) -> (Vec<MeshVertex>, IntersectionVertexMap) {
    let build_vertex = |p| {
        let normal = -implicit::gradient(density_func, p).normalize();
        MeshVertex {
            position: [p.x as f32, p.y as f32, p.z as f32],
            normal: [normal.x as f32, normal.y as f32, normal.z as f32],
        }
    };

    let mut mapping = IntersectionVertexMap::new();
    let mut vertices: Vec<MeshVertex> = Vec::new();

    let mut vertex_index: u32 = 0;
    for intersection in intersections {
        if let Some(intersection_position) = intersection {
            vertices.push(build_vertex(*intersection_position));
            mapping.push(Some(vertex_index));
            vertex_index += 1;
        } else {
            mapping.push(None);
        }
    }

    (vertices, mapping)
}

// For each cell in the grid evaluate edges specified in EDGE_INDICES
// and find the intersections points on them, if any
fn find_intersections(grid: &Grid) -> IntersectionContainer {
    let mut intersections = IntersectionContainer::new();
    // Loop over all points in the grid, for each point evaluate neighboring edges
    for z in 0..grid.depth {
        for y in 0..grid.height {
            for x in 0..grid.width {
                let base_cell_position = GridPosition { x, y, z };
                let cell_case = get_cell_case(grid, base_cell_position);
                let intersected_edges = EDGES_LOOKUP[cell_case];

                let base_cell = grid.get_cell(base_cell_position).unwrap();
                for edge_index in EDGE_INDICES.iter() {
                    let edge_index = *edge_index;
                    let is_edge_intersected = (intersected_edges & (1 << edge_index)) != 0;
                    if !is_edge_intersected {
                        intersections.push(None);
                        continue;
                    }

                    if let Some(edge_cell) = get_edge_end(grid, base_cell_position, edge_index) {
                        let intersection = get_intersection(base_cell, edge_cell);
                        intersections.push(intersection);
                    } else {
                        // Do not evaluate intersections for edges outside the grid
                        intersections.push(None);
                    }
                }
            }
        }
    }

    intersections
}

fn assemble_triangles(grid: &Grid, vertex_mapping: &IntersectionVertexMap) -> Vec<u32> {
    let mut indices: Vec<u32> = Vec::new();
    // Loop over the actual cubes, not individual grid vertices
    for z in 0..grid.depth - 1 {
        for y in 0..grid.height - 1 {
            for x in 0..grid.width - 1 {
                let grid_position = GridPosition::new(x, y, z);
                let edge_vertex_map =
                    get_edge_intersections_for_cell(grid, grid_position, vertex_mapping);

                let case = get_cell_case(grid, grid_position);
                let lookup_base = case * TRIANGLES * TRIANGLE_VERTICES;
                let mut edge_index = TRIANGLES_LOOKUP[lookup_base];
                let mut i = 0;
                while edge_index != EDGE_INVALID_INDEX && (i / TRIANGLE_VERTICES) < TRIANGLES {
                    indices.push(edge_vertex_map[edge_index as usize].unwrap());

                    i += 1;
                    edge_index = TRIANGLES_LOOKUP[lookup_base + i];
                }
            }
        }
    }

    indices
}

enum CellEdge {
    Back,
    Right,
    Up,
}

const CUBE_EDGES: usize = 12;
fn get_edge_intersections_for_cell(
    grid: &Grid,
    cell_position: GridPosition,
    vertex_mapping: &IntersectionVertexMap,
) -> [Option<u32>; CUBE_EDGES] {
    let get_cube_vertex_index = |add_x: usize, add_y: usize, add_z: usize| {
        let grid_cell_position = add(cell_position, add_x, add_y, add_z);
        let cell_index = grid.get_index_for(grid_cell_position);

        cell_index * INTERSECTION_STRIDE
    };

    let get_edge_index = |cube_vertex_index: usize, edge: CellEdge| {
        let edge_offset = match edge {
            CellEdge::Back => 0,
            CellEdge::Right => 1,
            CellEdge::Up => 2,
        };

        let edge_index = cube_vertex_index + edge_offset;
        vertex_mapping[edge_index]
    };

    // named after cube vertices
    // {[b]ottom | [t]op}{[l]eft | [r]ight}{[f]orward | [b]ack}
    let cube_blf = get_cube_vertex_index(0, 0, 0);
    let cube_brf = get_cube_vertex_index(1, 0, 0);
    let cube_tlf = get_cube_vertex_index(0, 1, 0);
    let cube_trf = get_cube_vertex_index(1, 1, 0);
    let cube_blb = get_cube_vertex_index(0, 0, 1);
    let cube_brb = get_cube_vertex_index(1, 0, 1);
    let cube_tlb = get_cube_vertex_index(0, 1, 1);

    let edge_intersections = [
        // Bottom _ edges clockwise
        get_edge_index(cube_blf, CellEdge::Back),
        get_edge_index(cube_blb, CellEdge::Right),
        get_edge_index(cube_brf, CellEdge::Back),
        get_edge_index(cube_blf, CellEdge::Right),
        // top _ edges colockwise
        get_edge_index(cube_tlf, CellEdge::Back),
        get_edge_index(cube_tlb, CellEdge::Right),
        get_edge_index(cube_trf, CellEdge::Back),
        get_edge_index(cube_tlf, CellEdge::Right),
        // | edges connecting bottom and top, clockwise
        get_edge_index(cube_blf, CellEdge::Up),
        get_edge_index(cube_blb, CellEdge::Up),
        get_edge_index(cube_brb, CellEdge::Up),
        get_edge_index(cube_brf, CellEdge::Up),
    ];

    edge_intersections
}

fn get_edge_end(grid: &Grid, edge_start: GridPosition, edge_index: u16) -> Option<GridPoint> {
    let end_position = match edge_index {
        0 => add(edge_start, 0, 0, 1),
        3 => add(edge_start, 1, 0, 0),
        8 => add(edge_start, 0, 1, 0),
        _ => todo!(),
    };

    grid.get_cell(end_position)
}

// TODO: interpolate points based on density
fn get_intersection(edge_start: GridPoint, edge_end: GridPoint) -> Intersection {
    return Some(edge_start.position.midpoint(edge_end.position));
}

const CUBE_VERTICES: u16 = 8;
// Assumes vertex index zero is left front bottom
const ADD_X: [u16; 4] = [2, 3, 6, 7];
const ADD_Y: [u16; 4] = [4, 5, 6, 7];
const ADD_Z: [u16; 4] = [1, 2, 5, 6];
fn get_cell_case(grid: &Grid, cell_index: GridPosition) -> usize {
    let mut base_cell = grid.get_cell(cell_index).unwrap();

    if let Some(lookup_index) = base_cell.case {
        return lookup_index;
    }

    let mut lookup_index: u32 = 0;
    for i in 0..CUBE_VERTICES {
        let grid_position = add(
            cell_index,
            if ADD_X.contains(&i) { 1 } else { 0 },
            if ADD_Y.contains(&i) { 1 } else { 0 },
            if ADD_Z.contains(&i) { 1 } else { 0 },
        );

        // If we go outside the grid, there is no edge..
        if let Some(cell) = grid.get_cell(grid_position) {
            if cell.density < SURFACE_LEVEL {
                lookup_index |= 1 << i;
            }
        }
    }

    let case = lookup_index as usize;
    base_cell.case = Some(case);
    return case;
}

#[derive(Clone, Copy)]
struct GridPoint {
    pub position: Position,
    pub density: Real,

    // Option because it is lazily evaluated, each point has a valid case value
    pub case: Option<usize>,
}

type GridPosition = Point3<usize>;
fn add(gp: GridPosition, x: usize, y: usize, z: usize) -> GridPosition {
    GridPosition {
        x: gp.x + x,
        y: gp.y + y,
        z: gp.z + z,
    }
}

struct Grid {
    data: Vec<GridPoint>,
    pub width: usize,
    pub height: usize,
    pub depth: usize,
}

impl Grid {
    pub fn new(support: Rectangle3D, density_function: &impl Fn(Position) -> Real) -> Self {
        let mut grid_data: Vec<GridPoint> = Vec::new();
        let depth_cells = (support.depth / CELL_SIZE) as usize + 1;
        let height_cells = (support.height / CELL_SIZE) as usize + 1;
        let width_cells = (support.width / CELL_SIZE) as usize + 1;

        // Create the grid 1 cell bigger in all dimensions
        // this way we have information about all points within the grid
        for z in 0..depth_cells {
            for y in 0..height_cells {
                for x in 0..width_cells {
                    let point_position = Position {
                        x: support.position.x + (x as Real) * CELL_SIZE,
                        y: support.position.y + (y as Real) * CELL_SIZE,
                        z: support.position.z + (z as Real) * CELL_SIZE,
                    };

                    let point_density = density_function(point_position);
                    grid_data.push(GridPoint {
                        position: point_position,
                        density: point_density,
                        case: None,
                    });
                }
            }
        }

        //println!("Grid cells: {}", grid_data.len());

        Grid {
            data: grid_data,
            width: width_cells,
            height: height_cells,
            depth: depth_cells,
        }
    }

    pub fn get_index_for(&self, pos: GridPosition) -> usize {
        pos.x + pos.y * self.width + (self.width * self.height) * pos.z
    }

    pub fn get_cell(&self, pos: GridPosition) -> Option<GridPoint> {
        if pos.x >= self.width || pos.y >= self.height || pos.z >= self.depth {
            return None;
        }

        let index = self.get_index_for(pos);
        //println!("pos {:?} -> inx {}", pos, index);
        Some(self.get_cell_by_index(index))
    }

    pub fn get_cell_by_index(&self, index: usize) -> GridPoint {
        self.data[index]
    }
}

#[derive(Clone, Copy)]
pub struct Rectangle3D {
    pub position: Position,
    pub width: Real,
    pub height: Real,
    pub depth: Real,
}

#[derive(Copy, Clone)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}
implement_vertex!(MeshVertex, position, normal);

const CASES: usize = 256;
const EDGES_LOOKUP: [u16; CASES] = [
    0x000, 0x109, 0x203, 0x30a, 0x406, 0x50f, 0x605, 0x70c, 0x80c, 0x905, 0xa0f, 0xb06, 0xc0a,
    0xd03, 0xe09, 0xf00, 0x190, 0x099, 0x393, 0x29a, 0x596, 0x49f, 0x795, 0x69c, 0x99c, 0x895,
    0xb9f, 0xa96, 0xd9a, 0xc93, 0xf99, 0xe90, 0x230, 0x339, 0x033, 0x13a, 0x636, 0x73f, 0x435,
    0x53c, 0xa3c, 0xb35, 0x83f, 0x936, 0xe3a, 0xf33, 0xc39, 0xd30, 0x3a0, 0x2a9, 0x1a3, 0x0aa,
    0x7a6, 0x6af, 0x5a5, 0x4ac, 0xbac, 0xaa5, 0x9af, 0x8a6, 0xfaa, 0xea3, 0xda9, 0xca0, 0x460,
    0x569, 0x663, 0x76a, 0x066, 0x16f, 0x265, 0x36c, 0xc6c, 0xd65, 0xe6f, 0xf66, 0x86a, 0x963,
    0xa69, 0xb60, 0x5f0, 0x4f9, 0x7f3, 0x6fa, 0x1f6, 0x0ff, 0x3f5, 0x2fc, 0xdfc, 0xcf5, 0xfff,
    0xef6, 0x9fa, 0x8f3, 0xbf9, 0xaf0, 0x650, 0x759, 0x453, 0x55a, 0x256, 0x35f, 0x055, 0x15c,
    0xe5c, 0xf55, 0xc5f, 0xd56, 0xa5a, 0xb53, 0x859, 0x950, 0x7c0, 0x6c9, 0x5c3, 0x4ca, 0x3c6,
    0x2cf, 0x1c5, 0x0cc, 0xfcc, 0xec5, 0xdcf, 0xcc6, 0xbca, 0xac3, 0x9c9, 0x8c0, 0x8c0, 0x9c9,
    0xac3, 0xbca, 0xcc6, 0xdcf, 0xec5, 0xfcc, 0x0cc, 0x1c5, 0x2cf, 0x3c6, 0x4ca, 0x5c3, 0x6c9,
    0x7c0, 0x950, 0x859, 0xb53, 0xa5a, 0xd56, 0xc5f, 0xf55, 0xe5c, 0x15c, 0x055, 0x35f, 0x256,
    0x55a, 0x453, 0x759, 0x650, 0xaf0, 0xbf9, 0x8f3, 0x9fa, 0xef6, 0xfff, 0xcf5, 0xdfc, 0x2fc,
    0x3f5, 0x0ff, 0x1f6, 0x6fa, 0x7f3, 0x4f9, 0x5f0, 0xb60, 0xa69, 0x963, 0x86a, 0xf66, 0xe6f,
    0xd65, 0xc6c, 0x36c, 0x265, 0x16f, 0x066, 0x76a, 0x663, 0x569, 0x460, 0xca0, 0xda9, 0xea3,
    0xfaa, 0x8a6, 0x9af, 0xaa5, 0xbac, 0x4ac, 0x5a5, 0x6af, 0x7a6, 0x0aa, 0x1a3, 0x2a9, 0x3a0,
    0xd30, 0xc39, 0xf33, 0xe3a, 0x936, 0x83f, 0xb35, 0xa3c, 0x53c, 0x435, 0x73f, 0x636, 0x13a,
    0x033, 0x339, 0x230, 0xe90, 0xf99, 0xc93, 0xd9a, 0xa96, 0xb9f, 0x895, 0x99c, 0x69c, 0x795,
    0x49f, 0x596, 0x29a, 0x393, 0x099, 0x190, 0xf00, 0xe09, 0xd03, 0xc0a, 0xb06, 0xa0f, 0x905,
    0x80c, 0x70c, 0x605, 0x50f, 0x406, 0x30a, 0x203, 0x109, 0x000,
];

const EDGE_INVALID_INDEX: i16 = -1;
const TRIANGLE_VERTICES: usize = 3;
const TRIANGLES: usize = 5;
const TRIANGLES_LOOKUP: [i16; CASES * TRIANGLES * 3] = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 8, 3, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, 0, 1, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 1, 8, 3, 9, 8,
    1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 1, 2, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, 0, 8, 3, 1, 2, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, 9, 2, 10, 0, 2, 9, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, 2, 8, 3, 2, 10, 8, 10, 9, 8, -1, -1, -1, -1, -1, -1, 3, 11, 2, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 11, 2, 8, 11, 0, -1, -1, -1, -1, -1, -1, -1, -1, -1, 1,
    9, 0, 2, 3, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, 1, 11, 2, 1, 9, 11, 9, 8, 11, -1, -1, -1,
    -1, -1, -1, 3, 10, 1, 11, 10, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 10, 1, 0, 8, 10, 8, 11,
    10, -1, -1, -1, -1, -1, -1, 3, 9, 0, 3, 11, 9, 11, 10, 9, -1, -1, -1, -1, -1, -1, 9, 8, 10, 10,
    8, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, 4, 7, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, 4, 3, 0, 7, 3, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 1, 9, 8, 4, 7, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, 4, 1, 9, 4, 7, 1, 7, 3, 1, -1, -1, -1, -1, -1, -1, 1, 2, 10, 8, 4, 7, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, 3, 4, 7, 3, 0, 4, 1, 2, 10, -1, -1, -1, -1, -1, -1, 9, 2, 10, 9, 0,
    2, 8, 4, 7, -1, -1, -1, -1, -1, -1, 2, 10, 9, 2, 9, 7, 2, 7, 3, 7, 9, 4, -1, -1, -1, 8, 4, 7,
    3, 11, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, 11, 4, 7, 11, 2, 4, 2, 0, 4, -1, -1, -1, -1, -1,
    -1, 9, 0, 1, 8, 4, 7, 2, 3, 11, -1, -1, -1, -1, -1, -1, 4, 7, 11, 9, 4, 11, 9, 11, 2, 9, 2, 1,
    -1, -1, -1, 3, 10, 1, 3, 11, 10, 7, 8, 4, -1, -1, -1, -1, -1, -1, 1, 11, 10, 1, 4, 11, 1, 0, 4,
    7, 11, 4, -1, -1, -1, 4, 7, 8, 9, 0, 11, 9, 11, 10, 11, 0, 3, -1, -1, -1, 4, 7, 11, 4, 11, 9,
    9, 11, 10, -1, -1, -1, -1, -1, -1, 9, 5, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 9,
    5, 4, 0, 8, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 5, 4, 1, 5, 0, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, 8, 5, 4, 8, 3, 5, 3, 1, 5, -1, -1, -1, -1, -1, -1, 1, 2, 10, 9, 5, 4, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, 3, 0, 8, 1, 2, 10, 4, 9, 5, -1, -1, -1, -1, -1, -1, 5, 2, 10, 5, 4, 2,
    4, 0, 2, -1, -1, -1, -1, -1, -1, 2, 10, 5, 3, 2, 5, 3, 5, 4, 3, 4, 8, -1, -1, -1, 9, 5, 4, 2,
    3, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 11, 2, 0, 8, 11, 4, 9, 5, -1, -1, -1, -1, -1, -1,
    0, 5, 4, 0, 1, 5, 2, 3, 11, -1, -1, -1, -1, -1, -1, 2, 1, 5, 2, 5, 8, 2, 8, 11, 4, 8, 5, -1,
    -1, -1, 10, 3, 11, 10, 1, 3, 9, 5, 4, -1, -1, -1, -1, -1, -1, 4, 9, 5, 0, 8, 1, 8, 10, 1, 8,
    11, 10, -1, -1, -1, 5, 4, 0, 5, 0, 11, 5, 11, 10, 11, 0, 3, -1, -1, -1, 5, 4, 8, 5, 8, 10, 10,
    8, 11, -1, -1, -1, -1, -1, -1, 9, 7, 8, 5, 7, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, 9, 3, 0,
    9, 5, 3, 5, 7, 3, -1, -1, -1, -1, -1, -1, 0, 7, 8, 0, 1, 7, 1, 5, 7, -1, -1, -1, -1, -1, -1, 1,
    5, 3, 3, 5, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, 9, 7, 8, 9, 5, 7, 10, 1, 2, -1, -1, -1, -1,
    -1, -1, 10, 1, 2, 9, 5, 0, 5, 3, 0, 5, 7, 3, -1, -1, -1, 8, 0, 2, 8, 2, 5, 8, 5, 7, 10, 5, 2,
    -1, -1, -1, 2, 10, 5, 2, 5, 3, 3, 5, 7, -1, -1, -1, -1, -1, -1, 7, 9, 5, 7, 8, 9, 3, 11, 2, -1,
    -1, -1, -1, -1, -1, 9, 5, 7, 9, 7, 2, 9, 2, 0, 2, 7, 11, -1, -1, -1, 2, 3, 11, 0, 1, 8, 1, 7,
    8, 1, 5, 7, -1, -1, -1, 11, 2, 1, 11, 1, 7, 7, 1, 5, -1, -1, -1, -1, -1, -1, 9, 5, 8, 8, 5, 7,
    10, 1, 3, 10, 3, 11, -1, -1, -1, 5, 7, 0, 5, 0, 9, 7, 11, 0, 1, 0, 10, 11, 10, 0, 11, 10, 0,
    11, 0, 3, 10, 5, 0, 8, 0, 7, 5, 7, 0, 11, 10, 5, 7, 11, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    10, 6, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 8, 3, 5, 10, 6, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, 9, 0, 1, 5, 10, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, 1, 8, 3, 1, 9, 8, 5,
    10, 6, -1, -1, -1, -1, -1, -1, 1, 6, 5, 2, 6, 1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 1, 6, 5,
    1, 2, 6, 3, 0, 8, -1, -1, -1, -1, -1, -1, 9, 6, 5, 9, 0, 6, 0, 2, 6, -1, -1, -1, -1, -1, -1, 5,
    9, 8, 5, 8, 2, 5, 2, 6, 3, 2, 8, -1, -1, -1, 2, 3, 11, 10, 6, 5, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, 11, 0, 8, 11, 2, 0, 10, 6, 5, -1, -1, -1, -1, -1, -1, 0, 1, 9, 2, 3, 11, 5, 10, 6, -1,
    -1, -1, -1, -1, -1, 5, 10, 6, 1, 9, 2, 9, 11, 2, 9, 8, 11, -1, -1, -1, 6, 3, 11, 6, 5, 3, 5, 1,
    3, -1, -1, -1, -1, -1, -1, 0, 8, 11, 0, 11, 5, 0, 5, 1, 5, 11, 6, -1, -1, -1, 3, 11, 6, 0, 3,
    6, 0, 6, 5, 0, 5, 9, -1, -1, -1, 6, 5, 9, 6, 9, 11, 11, 9, 8, -1, -1, -1, -1, -1, -1, 5, 10, 6,
    4, 7, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, 4, 3, 0, 4, 7, 3, 6, 5, 10, -1, -1, -1, -1, -1,
    -1, 1, 9, 0, 5, 10, 6, 8, 4, 7, -1, -1, -1, -1, -1, -1, 10, 6, 5, 1, 9, 7, 1, 7, 3, 7, 9, 4,
    -1, -1, -1, 6, 1, 2, 6, 5, 1, 4, 7, 8, -1, -1, -1, -1, -1, -1, 1, 2, 5, 5, 2, 6, 3, 0, 4, 3, 4,
    7, -1, -1, -1, 8, 4, 7, 9, 0, 5, 0, 6, 5, 0, 2, 6, -1, -1, -1, 7, 3, 9, 7, 9, 4, 3, 2, 9, 5, 9,
    6, 2, 6, 9, 3, 11, 2, 7, 8, 4, 10, 6, 5, -1, -1, -1, -1, -1, -1, 5, 10, 6, 4, 7, 2, 4, 2, 0, 2,
    7, 11, -1, -1, -1, 0, 1, 9, 4, 7, 8, 2, 3, 11, 5, 10, 6, -1, -1, -1, 9, 2, 1, 9, 11, 2, 9, 4,
    11, 7, 11, 4, 5, 10, 6, 8, 4, 7, 3, 11, 5, 3, 5, 1, 5, 11, 6, -1, -1, -1, 5, 1, 11, 5, 11, 6,
    1, 0, 11, 7, 11, 4, 0, 4, 11, 0, 5, 9, 0, 6, 5, 0, 3, 6, 11, 6, 3, 8, 4, 7, 6, 5, 9, 6, 9, 11,
    4, 7, 9, 7, 11, 9, -1, -1, -1, 10, 4, 9, 6, 4, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, 4, 10,
    6, 4, 9, 10, 0, 8, 3, -1, -1, -1, -1, -1, -1, 10, 0, 1, 10, 6, 0, 6, 4, 0, -1, -1, -1, -1, -1,
    -1, 8, 3, 1, 8, 1, 6, 8, 6, 4, 6, 1, 10, -1, -1, -1, 1, 4, 9, 1, 2, 4, 2, 6, 4, -1, -1, -1, -1,
    -1, -1, 3, 0, 8, 1, 2, 9, 2, 4, 9, 2, 6, 4, -1, -1, -1, 0, 2, 4, 4, 2, 6, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, 8, 3, 2, 8, 2, 4, 4, 2, 6, -1, -1, -1, -1, -1, -1, 10, 4, 9, 10, 6, 4, 11, 2,
    3, -1, -1, -1, -1, -1, -1, 0, 8, 2, 2, 8, 11, 4, 9, 10, 4, 10, 6, -1, -1, -1, 3, 11, 2, 0, 1,
    6, 0, 6, 4, 6, 1, 10, -1, -1, -1, 6, 4, 1, 6, 1, 10, 4, 8, 1, 2, 1, 11, 8, 11, 1, 9, 6, 4, 9,
    3, 6, 9, 1, 3, 11, 6, 3, -1, -1, -1, 8, 11, 1, 8, 1, 0, 11, 6, 1, 9, 1, 4, 6, 4, 1, 3, 11, 6,
    3, 6, 0, 0, 6, 4, -1, -1, -1, -1, -1, -1, 6, 4, 8, 11, 6, 8, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, 7, 10, 6, 7, 8, 10, 8, 9, 10, -1, -1, -1, -1, -1, -1, 0, 7, 3, 0, 10, 7, 0, 9, 10, 6, 7,
    10, -1, -1, -1, 10, 6, 7, 1, 10, 7, 1, 7, 8, 1, 8, 0, -1, -1, -1, 10, 6, 7, 10, 7, 1, 1, 7, 3,
    -1, -1, -1, -1, -1, -1, 1, 2, 6, 1, 6, 8, 1, 8, 9, 8, 6, 7, -1, -1, -1, 2, 6, 9, 2, 9, 1, 6, 7,
    9, 0, 9, 3, 7, 3, 9, 7, 8, 0, 7, 0, 6, 6, 0, 2, -1, -1, -1, -1, -1, -1, 7, 3, 2, 6, 7, 2, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, 2, 3, 11, 10, 6, 8, 10, 8, 9, 8, 6, 7, -1, -1, -1, 2, 0, 7, 2,
    7, 11, 0, 9, 7, 6, 7, 10, 9, 10, 7, 1, 8, 0, 1, 7, 8, 1, 10, 7, 6, 7, 10, 2, 3, 11, 11, 2, 1,
    11, 1, 7, 10, 6, 1, 6, 7, 1, -1, -1, -1, 8, 9, 6, 8, 6, 7, 9, 1, 6, 11, 6, 3, 1, 3, 6, 0, 9, 1,
    11, 6, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, 7, 8, 0, 7, 0, 6, 3, 11, 0, 11, 6, 0, -1, -1, -1,
    7, 11, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 7, 6, 11, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, 3, 0, 8, 11, 7, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 1, 9, 11, 7, 6,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 8, 1, 9, 8, 3, 1, 11, 7, 6, -1, -1, -1, -1, -1, -1, 10, 1,
    2, 6, 11, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, 1, 2, 10, 3, 0, 8, 6, 11, 7, -1, -1, -1, -1,
    -1, -1, 2, 9, 0, 2, 10, 9, 6, 11, 7, -1, -1, -1, -1, -1, -1, 6, 11, 7, 2, 10, 3, 10, 8, 3, 10,
    9, 8, -1, -1, -1, 7, 2, 3, 6, 2, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, 7, 0, 8, 7, 6, 0, 6, 2,
    0, -1, -1, -1, -1, -1, -1, 2, 7, 6, 2, 3, 7, 0, 1, 9, -1, -1, -1, -1, -1, -1, 1, 6, 2, 1, 8, 6,
    1, 9, 8, 8, 7, 6, -1, -1, -1, 10, 7, 6, 10, 1, 7, 1, 3, 7, -1, -1, -1, -1, -1, -1, 10, 7, 6, 1,
    7, 10, 1, 8, 7, 1, 0, 8, -1, -1, -1, 0, 3, 7, 0, 7, 10, 0, 10, 9, 6, 10, 7, -1, -1, -1, 7, 6,
    10, 7, 10, 8, 8, 10, 9, -1, -1, -1, -1, -1, -1, 6, 8, 4, 11, 8, 6, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, 3, 6, 11, 3, 0, 6, 0, 4, 6, -1, -1, -1, -1, -1, -1, 8, 6, 11, 8, 4, 6, 9, 0, 1, -1, -1,
    -1, -1, -1, -1, 9, 4, 6, 9, 6, 3, 9, 3, 1, 11, 3, 6, -1, -1, -1, 6, 8, 4, 6, 11, 8, 2, 10, 1,
    -1, -1, -1, -1, -1, -1, 1, 2, 10, 3, 0, 11, 0, 6, 11, 0, 4, 6, -1, -1, -1, 4, 11, 8, 4, 6, 11,
    0, 2, 9, 2, 10, 9, -1, -1, -1, 10, 9, 3, 10, 3, 2, 9, 4, 3, 11, 3, 6, 4, 6, 3, 8, 2, 3, 8, 4,
    2, 4, 6, 2, -1, -1, -1, -1, -1, -1, 0, 4, 2, 4, 6, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, 1, 9,
    0, 2, 3, 4, 2, 4, 6, 4, 3, 8, -1, -1, -1, 1, 9, 4, 1, 4, 2, 2, 4, 6, -1, -1, -1, -1, -1, -1, 8,
    1, 3, 8, 6, 1, 8, 4, 6, 6, 10, 1, -1, -1, -1, 10, 1, 0, 10, 0, 6, 6, 0, 4, -1, -1, -1, -1, -1,
    -1, 4, 6, 3, 4, 3, 8, 6, 10, 3, 0, 3, 9, 10, 9, 3, 10, 9, 4, 6, 10, 4, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, 4, 9, 5, 7, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 8, 3, 4, 9, 5, 11, 7, 6,
    -1, -1, -1, -1, -1, -1, 5, 0, 1, 5, 4, 0, 7, 6, 11, -1, -1, -1, -1, -1, -1, 11, 7, 6, 8, 3, 4,
    3, 5, 4, 3, 1, 5, -1, -1, -1, 9, 5, 4, 10, 1, 2, 7, 6, 11, -1, -1, -1, -1, -1, -1, 6, 11, 7, 1,
    2, 10, 0, 8, 3, 4, 9, 5, -1, -1, -1, 7, 6, 11, 5, 4, 10, 4, 2, 10, 4, 0, 2, -1, -1, -1, 3, 4,
    8, 3, 5, 4, 3, 2, 5, 10, 5, 2, 11, 7, 6, 7, 2, 3, 7, 6, 2, 5, 4, 9, -1, -1, -1, -1, -1, -1, 9,
    5, 4, 0, 8, 6, 0, 6, 2, 6, 8, 7, -1, -1, -1, 3, 6, 2, 3, 7, 6, 1, 5, 0, 5, 4, 0, -1, -1, -1, 6,
    2, 8, 6, 8, 7, 2, 1, 8, 4, 8, 5, 1, 5, 8, 9, 5, 4, 10, 1, 6, 1, 7, 6, 1, 3, 7, -1, -1, -1, 1,
    6, 10, 1, 7, 6, 1, 0, 7, 8, 7, 0, 9, 5, 4, 4, 0, 10, 4, 10, 5, 0, 3, 10, 6, 10, 7, 3, 7, 10, 7,
    6, 10, 7, 10, 8, 5, 4, 10, 4, 8, 10, -1, -1, -1, 6, 9, 5, 6, 11, 9, 11, 8, 9, -1, -1, -1, -1,
    -1, -1, 3, 6, 11, 0, 6, 3, 0, 5, 6, 0, 9, 5, -1, -1, -1, 0, 11, 8, 0, 5, 11, 0, 1, 5, 5, 6, 11,
    -1, -1, -1, 6, 11, 3, 6, 3, 5, 5, 3, 1, -1, -1, -1, -1, -1, -1, 1, 2, 10, 9, 5, 11, 9, 11, 8,
    11, 5, 6, -1, -1, -1, 0, 11, 3, 0, 6, 11, 0, 9, 6, 5, 6, 9, 1, 2, 10, 11, 8, 5, 11, 5, 6, 8, 0,
    5, 10, 5, 2, 0, 2, 5, 6, 11, 3, 6, 3, 5, 2, 10, 3, 10, 5, 3, -1, -1, -1, 5, 8, 9, 5, 2, 8, 5,
    6, 2, 3, 8, 2, -1, -1, -1, 9, 5, 6, 9, 6, 0, 0, 6, 2, -1, -1, -1, -1, -1, -1, 1, 5, 8, 1, 8, 0,
    5, 6, 8, 3, 8, 2, 6, 2, 8, 1, 5, 6, 2, 1, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, 1, 3, 6, 1, 6,
    10, 3, 8, 6, 5, 6, 9, 8, 9, 6, 10, 1, 0, 10, 0, 6, 9, 5, 0, 5, 6, 0, -1, -1, -1, 0, 3, 8, 5, 6,
    10, -1, -1, -1, -1, -1, -1, -1, -1, -1, 10, 5, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, 11, 5, 10, 7, 5, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, 11, 5, 10, 11, 7, 5, 8, 3, 0, -1,
    -1, -1, -1, -1, -1, 5, 11, 7, 5, 10, 11, 1, 9, 0, -1, -1, -1, -1, -1, -1, 10, 7, 5, 10, 11, 7,
    9, 8, 1, 8, 3, 1, -1, -1, -1, 11, 1, 2, 11, 7, 1, 7, 5, 1, -1, -1, -1, -1, -1, -1, 0, 8, 3, 1,
    2, 7, 1, 7, 5, 7, 2, 11, -1, -1, -1, 9, 7, 5, 9, 2, 7, 9, 0, 2, 2, 11, 7, -1, -1, -1, 7, 5, 2,
    7, 2, 11, 5, 9, 2, 3, 2, 8, 9, 8, 2, 2, 5, 10, 2, 3, 5, 3, 7, 5, -1, -1, -1, -1, -1, -1, 8, 2,
    0, 8, 5, 2, 8, 7, 5, 10, 2, 5, -1, -1, -1, 9, 0, 1, 5, 10, 3, 5, 3, 7, 3, 10, 2, -1, -1, -1, 9,
    8, 2, 9, 2, 1, 8, 7, 2, 10, 2, 5, 7, 5, 2, 1, 3, 5, 3, 7, 5, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, 0, 8, 7, 0, 7, 1, 1, 7, 5, -1, -1, -1, -1, -1, -1, 9, 0, 3, 9, 3, 5, 5, 3, 7, -1, -1, -1,
    -1, -1, -1, 9, 8, 7, 5, 9, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, 5, 8, 4, 5, 10, 8, 10, 11, 8,
    -1, -1, -1, -1, -1, -1, 5, 0, 4, 5, 11, 0, 5, 10, 11, 11, 3, 0, -1, -1, -1, 0, 1, 9, 8, 4, 10,
    8, 10, 11, 10, 4, 5, -1, -1, -1, 10, 11, 4, 10, 4, 5, 11, 3, 4, 9, 4, 1, 3, 1, 4, 2, 5, 1, 2,
    8, 5, 2, 11, 8, 4, 5, 8, -1, -1, -1, 0, 4, 11, 0, 11, 3, 4, 5, 11, 2, 11, 1, 5, 1, 11, 0, 2, 5,
    0, 5, 9, 2, 11, 5, 4, 5, 8, 11, 8, 5, 9, 4, 5, 2, 11, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, 2,
    5, 10, 3, 5, 2, 3, 4, 5, 3, 8, 4, -1, -1, -1, 5, 10, 2, 5, 2, 4, 4, 2, 0, -1, -1, -1, -1, -1,
    -1, 3, 10, 2, 3, 5, 10, 3, 8, 5, 4, 5, 8, 0, 1, 9, 5, 10, 2, 5, 2, 4, 1, 9, 2, 9, 4, 2, -1, -1,
    -1, 8, 4, 5, 8, 5, 3, 3, 5, 1, -1, -1, -1, -1, -1, -1, 0, 4, 5, 1, 0, 5, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, 8, 4, 5, 8, 5, 3, 9, 0, 5, 0, 3, 5, -1, -1, -1, 9, 4, 5, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, 4, 11, 7, 4, 9, 11, 9, 10, 11, -1, -1, -1, -1, -1, -1, 0, 8, 3, 4,
    9, 7, 9, 11, 7, 9, 10, 11, -1, -1, -1, 1, 10, 11, 1, 11, 4, 1, 4, 0, 7, 4, 11, -1, -1, -1, 3,
    1, 4, 3, 4, 8, 1, 10, 4, 7, 4, 11, 10, 11, 4, 4, 11, 7, 9, 11, 4, 9, 2, 11, 9, 1, 2, -1, -1,
    -1, 9, 7, 4, 9, 11, 7, 9, 1, 11, 2, 11, 1, 0, 8, 3, 11, 7, 4, 11, 4, 2, 2, 4, 0, -1, -1, -1,
    -1, -1, -1, 11, 7, 4, 11, 4, 2, 8, 3, 4, 3, 2, 4, -1, -1, -1, 2, 9, 10, 2, 7, 9, 2, 3, 7, 7, 4,
    9, -1, -1, -1, 9, 10, 7, 9, 7, 4, 10, 2, 7, 8, 7, 0, 2, 0, 7, 3, 7, 10, 3, 10, 2, 7, 4, 10, 1,
    10, 0, 4, 0, 10, 1, 10, 2, 8, 7, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, 4, 9, 1, 4, 1, 7, 7, 1,
    3, -1, -1, -1, -1, -1, -1, 4, 9, 1, 4, 1, 7, 0, 8, 1, 8, 7, 1, -1, -1, -1, 4, 0, 3, 7, 4, 3,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 4, 8, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 9,
    10, 8, 10, 11, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, 3, 0, 9, 3, 9, 11, 11, 9, 10, -1, -1, -1,
    -1, -1, -1, 0, 1, 10, 0, 10, 8, 8, 10, 11, -1, -1, -1, -1, -1, -1, 3, 1, 10, 11, 3, 10, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, 1, 2, 11, 1, 11, 9, 9, 11, 8, -1, -1, -1, -1, -1, -1, 3, 0, 9, 3,
    9, 11, 1, 2, 9, 2, 11, 9, -1, -1, -1, 0, 2, 11, 8, 0, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    3, 2, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 2, 3, 8, 2, 8, 10, 10, 8, 9, -1, -1,
    -1, -1, -1, -1, 9, 10, 2, 0, 9, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, 2, 3, 8, 2, 8, 10, 0, 1,
    8, 1, 10, 8, -1, -1, -1, 1, 10, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 1, 3, 8, 9,
    1, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 9, 1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, 0, 3, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1,
];
