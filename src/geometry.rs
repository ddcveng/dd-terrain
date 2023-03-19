use crate::infrastructure::vertex::Vertex;
use glium::{index::PrimitiveType, IndexBuffer, VertexBuffer, Display};

// colorful unit cube, each face has exclusive vertexes
pub fn cube_color_exclusive_vertex(display: &Display) -> (VertexBuffer<Vertex>, IndexBuffer<u32>){
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
        front_dl, front_dr, front_ul, front_ur,
        top_dl, top_dr, top_ul, top_ur,
        back_dl, back_dr, back_ul, back_ur,
        bottom_dl, bottom_dr, bottom_ul, bottom_ur,
        left_dl, left_dr, left_ul, left_ur,
        right_dl, right_dr, right_ul, right_ur,
    ];
    let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();

    // Faces share vertices, but the cube does not
    let indices = vec![
        0, 1, 2, 2, 1, 3,
        4, 5, 6, 6, 5, 7,
        8, 9, 10, 10, 9, 11,
        12, 13, 14, 14, 13, 15,
        16, 17, 18, 18, 17, 19,
        20, 21, 22, 22, 21, 23,
    ];
    let index_buffer = glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &indices).unwrap();

    (vertex_buffer, index_buffer)
}
