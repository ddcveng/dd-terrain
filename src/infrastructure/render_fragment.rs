use std::marker::PhantomData;

use glium::PolygonMode;
use glium::index::IndicesSource;
use glium::program::Program;
use glium::uniforms::Uniforms;
use glium::Surface;
use glium::VertexBuffer;

// TODO: will be made obsolete when builder will be type safe
#[derive(Debug)]
pub enum FragmentCreationError {
    NoGeometry,
}

// TODO: implement custom Uniforms type so I can manage it dynamically
pub struct RenderFragment<'a, T, I>
where
    T: Copy,
    //I: Into<IndicesSource<'a>> + 'a,
    I: 'a,
    IndicesSource<'a>: From<&'a I>,
{
    vertex_buffer: VertexBuffer<T>,
    indices: I,
    program: Program, // no compute shaders for now, separate entity
    //uniforms: Vec<dyn AsUniformValue>,
    _marker: PhantomData<&'a ()>,
}

impl<'a, T, I> RenderFragment<'a, T, I>
where
    T: Copy,
    //I: Into<IndicesSource<'a>>,
    I: 'a,
    IndicesSource<'a>: From<&'a I>,
{
    // TODO: check compatibility of uniforms and print warnings in debug mode
    pub fn render<U>(&'a self, target: &mut glium::Frame, uniforms: &U) 
    where U: Uniforms,
    {
        let params = glium::DrawParameters {
            backface_culling: glium::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        target
            .draw(
                &self.vertex_buffer,
                &self.indices,
                &self.program,
                uniforms,
                &params,
            )
            .unwrap();
    }

    pub fn render_instanced<U, D>(&'a self, target: &mut glium::Frame, uniforms: &U, instance_data: &VertexBuffer<D>, wireframe: bool) 
    where 
        U: Uniforms,
        D: Copy,
    {
        let polygon_mode = match wireframe {
            true => PolygonMode::Line,
            false => PolygonMode::Fill,
        };

        let draw_parameters = glium::DrawParameters {
            depth: glium::Depth { 
                test: glium::DepthTest::IfLess,
                write: true, 
                ..Default::default() 
            },
            backface_culling: glium::BackfaceCullingMode::CullClockwise,
            polygon_mode,
            ..Default::default()
        };

        target
            .draw(
                (&self.vertex_buffer, instance_data.per_instance().unwrap()),
                &self.indices,
                &self.program,
                uniforms,
                &draw_parameters,
            )
            .unwrap();
    }
}

// TODO: add marker type to represent build state so invalid state
// is not representable
pub struct RenderFragmentBuilder<'a, T, I/*, U*/>
where
    T: Copy,
    I: 'a,
    IndicesSource<'a>: From<&'a I>,
    //U: AsUniformValue,
{
    vertex_buffer: Option<VertexBuffer<T>>,
    indices: Option<I>,
    vertex_shader_source: Option<&'a str>,
    fragment_shader_source: Option<&'a str>,
    geometry_shader_source: Option<&'a str>,
    //uniforms: Option<UniformsStorage<'a, U, EmptyUniforms>>,
}

impl<'a, T, I/*, U*/> RenderFragmentBuilder<'a, T, I/*, U*/>
where
    T: Copy,
    I: 'a,
    IndicesSource<'a>: From<&'a I>,
    //U: AsUniformValue,
{
    pub fn new() -> Self {
        RenderFragmentBuilder {
            vertex_buffer: None,
            indices: None,
            vertex_shader_source: None,
            fragment_shader_source: None,
            geometry_shader_source: None,
            //uniforms: None,
        }
    }

    pub fn set_geometry(mut self, vertices: VertexBuffer<T>, indices: I) -> Self {
        self.vertex_buffer = Some(vertices);
        self.indices = Some(indices);

        self
    }

    pub fn set_vertex_shader(mut self, vertex_shader_source: &'a str) -> Self {
        self.vertex_shader_source = Some(vertex_shader_source);

        self
    }

    pub fn set_fragment_shader(mut self, fragment_shader_source: &'a str) -> Self {
        self.fragment_shader_source = Some(fragment_shader_source);

        self
    }

    pub fn set_geometry_shader(mut self, geometry_shader_source: &'a str) -> Self {
        self.geometry_shader_source = Some(geometry_shader_source);

        self
    }

//    pub fn set_uniforms(mut self, uniforms: UniformsStorage<'a, U, EmptyUniforms>) -> Self {
//        self.uniforms = Some(uniforms);
//
//        self
//    }

    pub fn build(
        self,
        display: &glium::Display,
    ) -> Result<RenderFragment<'a, T, I>, FragmentCreationError> {
        let vertex_buffer = self
            .vertex_buffer
            .ok_or(FragmentCreationError::NoGeometry)?;
        let indices = self.indices.ok_or(FragmentCreationError::NoGeometry)?;
        println!("1");

        let vertex_shader_source = self
            .vertex_shader_source
            .ok_or(FragmentCreationError::NoGeometry)?;
        println!("2");
        let fragment_shader_source = self
            .fragment_shader_source
            .ok_or(FragmentCreationError::NoGeometry)?;
        println!("3");

        let program_x = Program::from_source(
            display,
            vertex_shader_source,
            fragment_shader_source,
            self.geometry_shader_source,
        );

        println!("{:?}", program_x);
        println!("4");

        let program = program_x.or(Err(FragmentCreationError::NoGeometry))?;

        Ok(RenderFragment {
            vertex_buffer,
            indices,
            program,
            _marker: PhantomData::default(),
        })
    }
}
