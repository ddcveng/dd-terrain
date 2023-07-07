use crate::infrastructure::vertex::{TexturedVertex, Vertex};
use crate::model::common::BLOCK_TEXTURE_FRACTION;
use glium::{index::PrimitiveType, Display, IndexBuffer, VertexBuffer};

// colorful unit cube, each face has exclusive vertexes
#[allow(dead_code)]
pub fn cube_color_exclusive_vertex(display: &Display) -> (VertexBuffer<Vertex>, IndexBuffer<u32>) {
    // front face
    let color_red = [1.0, 0.0, 0.0];
    let front_normal = [0.0, 0.0, 1.0];
    let front_down_left = Vertex {
        position: [-0.5, -0.5, 0.5],
        color: color_red,
        normal: front_normal,
    };
    let front_down_right = Vertex {
        position: [0.5, -0.5, 0.5],
        color: color_red,
        normal: front_normal,
    };
    let front_up_left = Vertex {
        position: [-0.5, 0.5, 0.5],
        color: color_red,
        normal: front_normal,
    };
    let front_up_right = Vertex {
        position: [0.5, 0.5, 0.5],
        color: color_red,
        normal: front_normal,
    };

    // top face
    let color_green = [0.0, 1.0, 0.0];
    let top_normal = [0.0, 1.0, 0.0];
    let top_down_left = Vertex {
        position: [-0.5, 0.5, 0.5],
        color: color_green,
        normal: top_normal,
    };

    let top_down_right = Vertex {
        position: [0.5, 0.5, 0.5],
        color: color_green,
        normal: top_normal,
    };

    let top_up_left = Vertex {
        position: [-0.5, 0.5, -0.5],
        color: color_green,
        normal: top_normal,
    };
    let top_up_right = Vertex {
        position: [0.5, 0.5, -0.5],
        color: color_green,
        normal: top_normal,
    };

    // back face
    let color_blue = [0.0, 0.0, 1.0];
    let back_normal = [0.0, 0.0, -1.0];
    let back_down_left = Vertex {
        position: [0.5, -0.5, -0.5],
        color: color_blue,
        normal: back_normal,
    };
    let back_down_right = Vertex {
        position: [-0.5, -0.5, -0.5],
        color: color_blue,
        normal: back_normal,
    };
    let back_up_left = Vertex {
        position: [0.5, 0.5, -0.5],
        color: color_blue,
        normal: back_normal,
    };
    let back_up_right = Vertex {
        position: [-0.5, 0.5, -0.5],
        color: color_blue,
        normal: back_normal,
    };

    // bottom face
    let color_yellow = [0.5, 0.5, 0.0];
    let bottom_normal = [0.0, -1.0, 0.0];
    let bottom_down_left = Vertex {
        position: [-0.5, -0.5, -0.5],
        color: color_yellow,
        normal: bottom_normal,
    };
    let bottom_down_right = Vertex {
        position: [0.5, -0.5, -0.5],
        color: color_yellow,
        normal: bottom_normal,
    };
    let bottom_up_left = Vertex {
        position: [-0.5, -0.5, 0.5],
        color: color_yellow,
        normal: bottom_normal,
    };
    let bottom_up_right = Vertex {
        position: [0.5, -0.5, 0.5],
        color: color_yellow,
        normal: bottom_normal,
    };

    // left face
    let color_magenta = [0.5, 0.0, 0.5];
    let left_normal = [-1.0, 0.0, 0.0];
    let left_down_left = Vertex {
        position: [-0.5, -0.5, -0.5],
        color: color_magenta,
        normal: left_normal,
    };
    let left_down_right = Vertex {
        position: [-0.5, -0.5, 0.5],
        color: color_magenta,
        normal: left_normal,
    };
    let left_up_left = Vertex {
        position: [-0.5, 0.5, -0.5],
        color: color_magenta,
        normal: left_normal,
    };
    let left_up_right = Vertex {
        position: [-0.5, 0.5, 0.5],
        color: color_magenta,
        normal: left_normal,
    };

    // right face
    let color_cyan = [0.0, 0.5, 0.5];
    let right_normal = [1.0, 0.0, 0.0];
    let right_down_left = Vertex {
        position: [0.5, -0.5, 0.5],
        color: color_cyan,
        normal: right_normal,
    };
    let right_down_right = Vertex {
        position: [0.5, -0.5, -0.5],
        color: color_cyan,
        normal: right_normal,
    };
    let right_up_left = Vertex {
        position: [0.5, 0.5, 0.5],
        color: color_cyan,
        normal: right_normal,
    };
    let right_up_right = Vertex {
        position: [0.5, 0.5, -0.5],
        color: color_cyan,
        normal: right_normal,
    };

    let shape = vec![
        front_down_left,
        front_down_right,
        front_up_left,
        front_up_right,
        top_down_left,
        top_down_right,
        top_up_left,
        top_up_right,
        back_down_left,
        back_down_right,
        back_up_left,
        back_up_right,
        bottom_down_left,
        bottom_down_right,
        bottom_up_left,
        bottom_up_right,
        left_down_left,
        left_down_right,
        left_up_left,
        left_up_right,
        right_down_left,
        right_down_right,
        right_up_left,
        right_up_right,
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
    let front_down_left = TexturedVertex {
        position: [-0.5, -0.5, 0.5],
        normal: front_normal,
        texture_coordinates: [0.0, 0.0],
    };
    let front_down_right = TexturedVertex {
        position: [0.5, -0.5, 0.5],
        normal: front_normal,
        texture_coordinates: [TEXTURE_END, 0.0],
    };
    let front_up_left = TexturedVertex {
        position: [-0.5, 0.5, 0.5],
        normal: front_normal,
        texture_coordinates: [0.0, TEXTURE_END],
    };
    let front_up_right = TexturedVertex {
        position: [0.5, 0.5, 0.5],
        normal: front_normal,
        texture_coordinates: [TEXTURE_END, TEXTURE_END],
    };

    // top face
    let top_normal = [0.0, 1.0, 0.0];
    let top_down_left = TexturedVertex {
        position: [-0.5, 0.5, 0.5],
        normal: top_normal,
        texture_coordinates: [0.0, 0.0],
    };
    let top_down_right = TexturedVertex {
        position: [0.5, 0.5, 0.5],
        normal: top_normal,
        texture_coordinates: [TEXTURE_END, 0.0],
    };
    let top_up_left = TexturedVertex {
        position: [-0.5, 0.5, -0.5],
        normal: top_normal,
        texture_coordinates: [0.0, TEXTURE_END],
    };
    let top_up_right = TexturedVertex {
        position: [0.5, 0.5, -0.5],
        normal: top_normal,
        texture_coordinates: [TEXTURE_END, TEXTURE_END],
    };

    // back face
    let back_normal = [0.0, 0.0, -1.0];
    let back_down_left = TexturedVertex {
        position: [0.5, -0.5, -0.5],
        normal: back_normal,
        texture_coordinates: [0.0, 0.0],
    };
    let back_down_right = TexturedVertex {
        position: [-0.5, -0.5, -0.5],
        normal: back_normal,
        texture_coordinates: [TEXTURE_END, 0.0],
    };
    let back_up_left = TexturedVertex {
        position: [0.5, 0.5, -0.5],
        normal: back_normal,
        texture_coordinates: [0.0, TEXTURE_END],
    };
    let back_up_right = TexturedVertex {
        position: [-0.5, 0.5, -0.5],
        normal: back_normal,
        texture_coordinates: [TEXTURE_END, TEXTURE_END],
    };

    // bottom face
    let bottom_normal = [0.0, -1.0, 0.0];
    let bottom_down_left = TexturedVertex {
        position: [-0.5, -0.5, -0.5],
        normal: bottom_normal,
        texture_coordinates: [0.0, 0.0],
    };
    let bottom_down_right = TexturedVertex {
        position: [0.5, -0.5, -0.5],
        normal: bottom_normal,
        texture_coordinates: [TEXTURE_END, 0.0],
    };
    let bottom_up_left = TexturedVertex {
        position: [-0.5, -0.5, 0.5],
        normal: bottom_normal,
        texture_coordinates: [0.0, TEXTURE_END],
    };
    let bottom_up_right = TexturedVertex {
        position: [0.5, -0.5, 0.5],
        normal: bottom_normal,
        texture_coordinates: [TEXTURE_END, TEXTURE_END],
    };

    // left face
    let left_normal = [-1.0, 0.0, 0.0];
    let left_down_left = TexturedVertex {
        position: [-0.5, -0.5, -0.5],
        normal: left_normal,
        texture_coordinates: [0.0, 0.0],
    };
    let left_down_right = TexturedVertex {
        position: [-0.5, -0.5, 0.5],
        normal: left_normal,
        texture_coordinates: [TEXTURE_END, 0.0],
    };
    let left_up_left = TexturedVertex {
        position: [-0.5, 0.5, -0.5],
        normal: left_normal,
        texture_coordinates: [0.0, TEXTURE_END],
    };
    let left_up_right = TexturedVertex {
        position: [-0.5, 0.5, 0.5],
        normal: left_normal,
        texture_coordinates: [TEXTURE_END, TEXTURE_END],
    };

    // right face
    let right_normal = [1.0, 0.0, 0.0];
    let right_down_left = TexturedVertex {
        position: [0.5, -0.5, 0.5],
        normal: right_normal,
        texture_coordinates: [0.0, 0.0],
    };
    let right_down_right = TexturedVertex {
        position: [0.5, -0.5, -0.5],
        normal: right_normal,
        texture_coordinates: [TEXTURE_END, 0.0],
    };
    let right_up_left = TexturedVertex {
        position: [0.5, 0.5, 0.5],
        normal: right_normal,
        texture_coordinates: [0.0, TEXTURE_END],
    };
    let right_up_right = TexturedVertex {
        position: [0.5, 0.5, -0.5],
        normal: right_normal,
        texture_coordinates: [TEXTURE_END, TEXTURE_END],
    };

    let shape = vec![
        front_down_left,
        front_down_right,
        front_up_left,
        front_up_right,
        top_down_left,
        top_down_right,
        top_up_left,
        top_up_right,
        back_down_left,
        back_down_right,
        back_up_left,
        back_up_right,
        bottom_down_left,
        bottom_down_right,
        bottom_up_left,
        bottom_up_right,
        left_down_left,
        left_down_right,
        left_up_left,
        left_up_right,
        right_down_left,
        right_down_right,
        right_up_left,
        right_up_right,
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
