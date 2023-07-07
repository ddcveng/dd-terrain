use glium::{index::IndicesSource, uniforms::Uniforms, DrawParameters, Frame, VertexBuffer};

use crate::infrastructure::render_fragment::RenderFragment;

// Represents a single render pass
// with support for instancing
//
// TODO: The only benefit of this abstraction is that it can store instance data along with the
// fragment. RenderFragment can now support both instanced and non instanced versions. Consider
// creating InstancedRenderFragment which will hold its instance data and this can then be removed
pub struct RenderPass<'a, D, T, I>
where
    D: Copy,
    T: Copy,
    I: 'a,
    IndicesSource<'a>: From<&'a I>,
{
    pub fragment: RenderFragment<'a, T, I>,
    pub instance_data: Option<VertexBuffer<D>>,
}

impl<'a, D, T, I> RenderPass<'a, D, T, I>
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
        RenderPass {
            fragment,
            instance_data: Some(instance_data),
        }
    }

    pub fn update_instance_data(&mut self, instance_data: VertexBuffer<D>) {
        self.instance_data = Some(instance_data);
    }

    pub fn execute<U>(
        &'a self,
        target: &mut Frame,
        uniforms: &U,
        draw_parameters: Option<DrawParameters>,
    ) where
        U: Uniforms,
    {
        if let Some(instance_data) = &self.instance_data {
            self.fragment
                .render_instanced(target, uniforms, instance_data, draw_parameters);
        } else {
            self.fragment.render(target, uniforms, draw_parameters);
        }
    }
}

// Dummy type used as D type when no instancing is required
#[derive(Clone, Copy)]
pub struct NoInstance {}

impl<'a, T, I> RenderPass<'a, NoInstance, T, I>
where
    T: Copy,
    I: 'a,
    IndicesSource<'a>: From<&'a I>,
{
    pub fn new(fragment: RenderFragment<'a, T, I>) -> Self {
        RenderPass {
            fragment,
            instance_data: None,
        }
    }
}
