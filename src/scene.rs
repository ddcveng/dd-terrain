use glium::{index::IndicesSource, VertexBuffer};

use crate::infrastructure::render_fragment::RenderFragment;

pub struct Scene<'a, D, T, I>
where
    D: Copy,
    T: Copy,
    I: 'a,
    IndicesSource<'a>: From<&'a I>,
{
    pub fragment: RenderFragment<'a, T, I>,
    pub instance_data: Option<VertexBuffer<D>>,
}

impl<'a, D, T, I> Scene<'a, D, T, I>
where
    D: Copy,
    T: Copy,
    I: 'a,
    IndicesSource<'a>: From<&'a I>,
{
    pub fn new_instanced(
        fragment: RenderFragment<'a, T, I>,
        instance_data: VertexBuffer<D>,
    ) -> Self {
        Scene {
            fragment,
            instance_data: Some(instance_data),
        }
    }

    pub fn update_instance_data(&mut self, instance_data: VertexBuffer<D>) {
        self.instance_data = Some(instance_data);
    }
}

// Dummy type used as D type when no instancing is required
#[derive(Clone, Copy)]
pub struct NoInstance {}

impl<'a, T, I> Scene<'a, NoInstance, T, I>
where
    T: Copy,
    I: 'a,
    IndicesSource<'a>: From<&'a I>,
{
    pub fn new(fragment: RenderFragment<'a, T, I>) -> Self {
        Scene {
            fragment,
            instance_data: None,
        }
    }
}
