use crate::infrastructure::vertex::{TexturedVertex, Vertex};
use crate::model::common::BLOCK_TEXTURE_FRACTION;
use glium::{index::PrimitiveType, Display, IndexBuffer, VertexBuffer};

// colorful unit cube, each face has exclusive vertexes
pub fn cube_color_exclusive_vertex(display: &Display) -> (VertexBuffer<Vertex>, IndexBuffer<u32>) {
    // front face
    let color_red = [1.0, 0.0, 0.0];
    let front_normal = [0.0, 0.0, 1.0];
    let front_dl = Vertex {
        position: [-0.5, -0.5, 0.5],
        color: color_red,
        normal: front_normal,
    };
    let front_dr = Vertex {
        position: [0.5, -0.5, 0.5],
        color: color_red,
        normal: front_normal,
    };
    let front_ul = Vertex {
        position: [-0.5, 0.5, 0.5],
        color: color_red,
        normal: front_normal,
    };
    let front_ur = Vertex {
        position: [0.5, 0.5, 0.5],
        color: color_red,
        normal: front_normal,
    };

    // top face
    let color_green = [0.0, 1.0, 0.0];
    let top_normal = [0.0, 1.0, 0.0];
    let top_dl = Vertex {
        position: [-0.5, 0.5, 0.5],
        color: color_green,
        normal: top_normal,
    };

    let top_dr = Vertex {
        position: [0.5, 0.5, 0.5],
        color: color_green,
        normal: top_normal,
    };

    let top_ul = Vertex {
        position: [-0.5, 0.5, -0.5],
        color: color_green,
        normal: top_normal,
    };
    let top_ur = Vertex {
        position: [0.5, 0.5, -0.5],
        color: color_green,
        normal: top_normal,
    };

    // back face
    let color_blue = [0.0, 0.0, 1.0];
    let back_normal = [0.0, 0.0, -1.0];
    let back_dl = Vertex {
        position: [0.5, -0.5, -0.5],
        color: color_blue,
        normal: back_normal,
    };
    let back_dr = Vertex {
        position: [-0.5, -0.5, -0.5],
        color: color_blue,
        normal: back_normal,
    };
    let back_ul = Vertex {
        position: [0.5, 0.5, -0.5],
        color: color_blue,
        normal: back_normal,
    };
    let back_ur = Vertex {
        position: [-0.5, 0.5, -0.5],
        color: color_blue,
        normal: back_normal,
    };

    // bottom face
    let color_yellow = [0.5, 0.5, 0.0];
    let bottom_normal = [0.0, -1.0, 0.0];
    let bottom_dl = Vertex {
        position: [-0.5, -0.5, -0.5],
        color: color_yellow,
        normal: bottom_normal,
    };
    let bottom_dr = Vertex {
        position: [0.5, -0.5, -0.5],
        color: color_yellow,
        normal: bottom_normal,
    };
    let bottom_ul = Vertex {
        position: [-0.5, -0.5, 0.5],
        color: color_yellow,
        normal: bottom_normal,
    };
    let bottom_ur = Vertex {
        position: [0.5, -0.5, 0.5],
        color: color_yellow,
        normal: bottom_normal,
    };

    // left face
    let color_magenta = [0.5, 0.0, 0.5];
    let left_normal = [-1.0, 0.0, 0.0];
    let left_dl = Vertex {
        position: [-0.5, -0.5, -0.5],
        color: color_magenta,
        normal: left_normal,
    };
    let left_dr = Vertex {
        position: [-0.5, -0.5, 0.5],
        color: color_magenta,
        normal: left_normal,
    };
    let left_ul = Vertex {
        position: [-0.5, 0.5, -0.5],
        color: color_magenta,
        normal: left_normal,
    };
    let left_ur = Vertex {
        position: [-0.5, 0.5, 0.5],
        color: color_magenta,
        normal: left_normal,
    };

    // right face
    let color_cyan = [0.0, 0.5, 0.5];
    let right_normal = [1.0, 0.0, 0.0];
    let right_dl = Vertex {
        position: [0.5, -0.5, 0.5],
        color: color_cyan,
        normal: right_normal,
    };
    let right_dr = Vertex {
        position: [0.5, -0.5, -0.5],
        color: color_cyan,
        normal: right_normal,
    };
    let right_ul = Vertex {
        position: [0.5, 0.5, 0.5],
        color: color_cyan,
        normal: right_normal,
    };
    let right_ur = Vertex {
        position: [0.5, 0.5, -0.5],
        color: color_cyan,
        normal: right_normal,
    };

    let shape = vec![
        front_dl, front_dr, front_ul, front_ur, top_dl, top_dr, top_ul, top_ur, back_dl, back_dr,
        back_ul, back_ur, bottom_dl, bottom_dr, bottom_ul, bottom_ur, left_dl, left_dr, left_ul,
        left_ur, right_dl, right_dr, right_ul, right_ur,
    ];
    let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();

    // Faces share vertices, but the cube does not
    let indices = vec![
        0, 1, 2, 2, 1, 3, 4, 5, 6, 6, 5, 7, 8, 9, 10, 10, 9, 11, 12, 13, 14, 14, 13, 15, 16, 17,
        18, 18, 17, 19, 20, 21, 22, 22, 21, 23,
    ];
    let index_buffer =
        glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &indices).unwrap();

    (vertex_buffer, index_buffer)
}

const TEXTURE_END: f32 = BLOCK_TEXTURE_FRACTION;
pub fn cube_textured_exclusive_vertex(
    display: &Display,
) -> (VertexBuffer<TexturedVertex>, IndexBuffer<u32>) {
    // front face
    let front_normal = [0.0, 0.0, 1.0];
    let front_dl = TexturedVertex {
        position: [-0.5, -0.5, 0.5],
        normal: front_normal,
        texture_coordinates: [0.0, 0.0],
    };
    let front_dr = TexturedVertex {
        position: [0.5, -0.5, 0.5],
        normal: front_normal,
        texture_coordinates: [TEXTURE_END, 0.0],
    };
    let front_ul = TexturedVertex {
        position: [-0.5, 0.5, 0.5],
        normal: front_normal,
        texture_coordinates: [0.0, TEXTURE_END],
    };
    let front_ur = TexturedVertex {
        position: [0.5, 0.5, 0.5],
        normal: front_normal,
        texture_coordinates: [TEXTURE_END, TEXTURE_END],
    };

    // top face
    let top_normal = [0.0, 1.0, 0.0];
    let top_dl = TexturedVertex {
        position: [-0.5, 0.5, 0.5],
        normal: top_normal,
        texture_coordinates: [0.0, 0.0],
    };
    let top_dr = TexturedVertex {
        position: [0.5, 0.5, 0.5],
        normal: top_normal,
        texture_coordinates: [TEXTURE_END, 0.0],
    };
    let top_ul = TexturedVertex {
        position: [-0.5, 0.5, -0.5],
        normal: top_normal,
        texture_coordinates: [0.0, TEXTURE_END],
    };
    let top_ur = TexturedVertex {
        position: [0.5, 0.5, -0.5],
        normal: top_normal,
        texture_coordinates: [TEXTURE_END, TEXTURE_END],
    };

    // back face
    let back_normal = [0.0, 0.0, -1.0];
    let back_dl = TexturedVertex {
        position: [0.5, -0.5, -0.5],
        normal: back_normal,
        texture_coordinates: [0.0, 0.0],
    };
    let back_dr = TexturedVertex {
        position: [-0.5, -0.5, -0.5],
        normal: back_normal,
        texture_coordinates: [TEXTURE_END, 0.0],
    };
    let back_ul = TexturedVertex {
        position: [0.5, 0.5, -0.5],
        normal: back_normal,
        texture_coordinates: [0.0, TEXTURE_END],
    };
    let back_ur = TexturedVertex {
        position: [-0.5, 0.5, -0.5],
        normal: back_normal,
        texture_coordinates: [TEXTURE_END, TEXTURE_END],
    };

    // bottom face
    let bottom_normal = [0.0, -1.0, 0.0];
    let bottom_dl = TexturedVertex {
        position: [-0.5, -0.5, -0.5],
        normal: bottom_normal,
        texture_coordinates: [0.0, 0.0],
    };
    let bottom_dr = TexturedVertex {
        position: [0.5, -0.5, -0.5],
        normal: bottom_normal,
        texture_coordinates: [TEXTURE_END, 0.0],
    };
    let bottom_ul = TexturedVertex {
        position: [-0.5, -0.5, 0.5],
        normal: bottom_normal,
        texture_coordinates: [0.0, TEXTURE_END],
    };
    let bottom_ur = TexturedVertex {
        position: [0.5, -0.5, 0.5],
        normal: bottom_normal,
        texture_coordinates: [TEXTURE_END, TEXTURE_END],
    };

    // left face
    let left_normal = [-1.0, 0.0, 0.0];
    let left_dl = TexturedVertex {
        position: [-0.5, -0.5, -0.5],
        normal: left_normal,
        texture_coordinates: [0.0, 0.0],
    };
    let left_dr = TexturedVertex {
        position: [-0.5, -0.5, 0.5],
        normal: left_normal,
        texture_coordinates: [TEXTURE_END, 0.0],
    };
    let left_ul = TexturedVertex {
        position: [-0.5, 0.5, -0.5],
        normal: left_normal,
        texture_coordinates: [0.0, TEXTURE_END],
    };
    let left_ur = TexturedVertex {
        position: [-0.5, 0.5, 0.5],
        normal: left_normal,
        texture_coordinates: [TEXTURE_END, TEXTURE_END],
    };

    // right face
    let right_normal = [1.0, 0.0, 0.0];
    let right_dl = TexturedVertex {
        position: [0.5, -0.5, 0.5],
        normal: right_normal,
        texture_coordinates: [0.0, 0.0],
    };
    let right_dr = TexturedVertex {
        position: [0.5, -0.5, -0.5],
        normal: right_normal,
        texture_coordinates: [TEXTURE_END, 0.0],
    };
    let right_ul = TexturedVertex {
        position: [0.5, 0.5, 0.5],
        normal: right_normal,
        texture_coordinates: [0.0, TEXTURE_END],
    };
    let right_ur = TexturedVertex {
        position: [0.5, 0.5, -0.5],
        normal: right_normal,
        texture_coordinates: [TEXTURE_END, TEXTURE_END],
    };

    let shape = vec![
        front_dl, front_dr, front_ul, front_ur, top_dl, top_dr, top_ul, top_ur, back_dl, back_dr,
        back_ul, back_ur, bottom_dl, bottom_dr, bottom_ul, bottom_ur, left_dl, left_dr, left_ul,
        left_ur, right_dl, right_dr, right_ul, right_ur,
    ];
    let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();

    // Faces share vertices, but the cube does not
    let indices = vec![
        0, 1, 2, 2, 1, 3, 4, 5, 6, 6, 5, 7, 8, 9, 10, 10, 9, 11, 12, 13, 14, 14, 13, 15, 16, 17,
        18, 18, 17, 19, 20, 21, 22, 22, 21, 23,
    ];
    let index_buffer =
        glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &indices).unwrap();

    (vertex_buffer, index_buffer)
}
