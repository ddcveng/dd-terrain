use glium::implement_vertex;

#[derive(Clone, Copy)]
pub struct Vertex2D {
    pub position: [f32; 2],
}
implement_vertex!(Vertex2D, position);

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
}
implement_vertex!(Vertex, position, color, normal);

#[derive(Clone, Copy)]
pub struct TexturedVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texture_coordinates: [f32; 2],
}
implement_vertex!(TexturedVertex, position, normal, texture_coordinates);
