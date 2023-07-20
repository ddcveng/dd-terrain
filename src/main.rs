use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::index::IndicesSource;
use glium::texture::SrgbTexture2d;
use glium::{uniform, Display, Frame, IndexBuffer, Surface};

use glium::glutin::event::VirtualKeyCode;
use glium::glutin::window::CursorGrabMode;
use glium::glutin::window::Window;

use array_init::array_init;
use cgmath::{Matrix4, Vector3};

mod imgui_wrapper;
use imgui_wrapper::{ImguiWrapper, SmoothMeshOptions, UIWindowBuilder};

mod minecraft;

mod camera;
use camera::Camera;

mod geometry;

mod infrastructure;
use infrastructure::input::{self, InputAction, InputConsumer};
use infrastructure::render_fragment::RenderFragmentBuilder;
use infrastructure::texture::texture_loader::texture_from_file;
use infrastructure::{RenderState, RenderingMode};
use minecraft::get_minecraft_chunk_position;

mod model;
use model::discrete::World;
use model::implicit::smooth::{get_density, get_smooth_normal};
use model::polygonize::{MeshVertex, PolygonizationOptions};
use model::{discrete, Real};

mod config;
mod scene;
use scene::{NoInstance, RenderPass};

mod macros;

const DISCRETE_VS: &str = include_str!("shaders/discrete_vs.glsl");
const DISCRETE_FS: &str = include_str!("shaders/discrete_fs.glsl");
const IMPLICIT_VS: &str = include_str!("shaders/implicit_vs.glsl");
const IMPLICIT_FS: &str = include_str!("shaders/implicit_fs.glsl");

fn main() {
    let (event_loop, display) = create_window();

    let block_pallette = texture_from_file("block-palette-tiling.png", &display);

    let mut controls = SmoothMeshOptions::default();
    let mut polygonization_options = controls.into();

    let mut world = discrete::World::new(config::SPAWN_POINT);
    world.dispatch_mesh_builder(polygonization_options);

    let mut camera = create_camera(display.get_framebuffer_dimensions());

    let mut rigid_scene = create_rigid_scene(&world, &display);
    let mut discrete_scene = create_discrete_scene(&world, &display);
    let mut implicit_scene = create_implicit_scene(&world, &display);

    let mut imgui_data = ImguiWrapper::new(&display);

    let mut render_state = RenderState::new();
    let mut actions: Vec<InputAction> = Vec::new();

    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(_) => {
            actions.clear();
            render_state.timing.record_frame();
        }
        Event::MainEventsCleared => {
            let gl_window = display.gl_window();
            let Some(new_state) = create_state(&actions, render_state, gl_window.window()) else {
                *control_flow = ControlFlow::Exit;
                return;
            };
            render_state = new_state;

            if controls.apply {
                polygonization_options = controls.into();
                world.rebuild_all_meshes(polygonization_options);

                controls.apply = false;
            }

            imgui_data.prepare(gl_window.window(), render_state.timing.delta_time);

            for action in &actions {
                camera.consume(action, &render_state);
            }

            camera.update(render_state.timing.delta_time.as_secs_f64());

            let update_geometry = config::DYNAMIC_WORLD
                && world.update_chunk_data(camera.get_position(), polygonization_options);

            if update_geometry {
                let instance_positions = {
                    let blocks = world.get_surface_block_data();
                    glium::vertex::VertexBuffer::new(&display, &blocks).unwrap()
                };
                discrete_scene.update_instance_data(instance_positions);

                let rigid_positions = {
                    let rigid_blocks = world.get_rigid_blocks_data();
                    glium::vertex::VertexBuffer::new(&display, &rigid_blocks).unwrap()
                };
                rigid_scene.update_instance_data(rigid_positions);
            }

            let update_implicit_scene = world.update_smooth_mesh();
            if update_implicit_scene {
                implicit_scene = create_implicit_scene(&world, &display);
            }

            gl_window.window().request_redraw();
        }
        Event::RedrawRequested(_) => {
            // Setup for drawing
            let gl_window = display.gl_window();
            let mut target = display.draw();

            // Clear window
            target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
            target.clear_depth(1.0);

            // Draw Scene
            match render_state.render_mode {
                RenderingMode::Discrete => render_world(
                    &discrete_scene,
                    &mut target,
                    &camera,
                    &render_state,
                    &block_pallette,
                ),
                RenderingMode::Implicit => {
                    if config::FILTER_RIGID {
                        // render rigid blocks
                        render_world(
                            &rigid_scene,
                            &mut target,
                            &camera,
                            &render_state,
                            &block_pallette,
                        );
                    }
                    // render smooth terrain
                    render_world(
                        &implicit_scene,
                        &mut target,
                        &camera,
                        &render_state,
                        &block_pallette,
                    );
                }
            }

            // Draw ui last so it shows on top of everything
            let statistics_menu_builder =
                get_statistics_menu_builder(&render_state, &camera, &world, polygonization_options);
            let controls_menu = get_controls_menu_builder();

            imgui_data.add_window(statistics_menu_builder);
            imgui_data.add_window(controls_menu);
            imgui_data
                .render_frame(gl_window.window(), &mut target, &mut controls)
                .expect("Failed to render imgui ui!");

            // Finish building the frame and swap buffers
            target.finish().expect("Failed to swap buffers");
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = ControlFlow::Exit,

        event => {
            let gl_window = display.gl_window();
            imgui_data.handle_event(gl_window.window(), &event);

            if let Some(action) = input::translate_event(event) {
                actions.push(action);
            }
        }
    });
}

fn to_uniform_matrix(matrix: &Matrix4<Real>) -> [[f32; 4]; 4] {
    array_init(|i| array_init(|j| matrix[i][j] as f32))
}

fn render_world<'a, D, T, I>(
    render_pass: &'a RenderPass<'a, D, T, I>,
    target: &mut Frame,
    camera: &Camera,
    state: &RenderState,
    texture: &SrgbTexture2d,
) -> ()
where
    D: Copy,
    T: Copy,
    I: 'a,
    IndicesSource<'a>: From<&'a I>,
{
    let camera_position = camera.get_position();
    let sun_position = [
        (camera_position.x + 200.0) as f32,
        (camera_position.y + 300.0) as f32,
        (camera_position.z + 200.0) as f32,
    ];

    let model: [[f32; 4]; 4] = cgmath::Matrix4::from_scale(1.0).into();
    let projection: [[f32; 4]; 4] = to_uniform_matrix(&camera.projection);
    let view: [[f32; 4]; 4] = to_uniform_matrix(&camera.world_to_view);

    let uni = uniform! {
        projection: projection,
        view: view,
        model: model,
        block_pallette: texture.sampled()
            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
            .wrap_function(glium::uniforms::SamplerWrapFunction::BorderClamp),
        sun_position: sun_position,
    };

    let polygon_mode = match state.render_wireframe {
        true => glium::PolygonMode::Line,
        false => glium::PolygonMode::Fill,
    };
    let draw_parameters = glium::DrawParameters {
        backface_culling: glium::BackfaceCullingMode::CullClockwise,
        polygon_mode,
        depth: glium::Depth {
            test: glium::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        ..Default::default()
    };

    render_pass.execute(target, &uni, Some(draw_parameters));
}

fn get_controls_menu_builder() -> UIWindowBuilder {
    let builder = move |ui: &imgui::Ui, controls: &mut SmoothMeshOptions| {
        ui.window("controls")
            .size([300.0, 150.0], imgui::Condition::FirstUseEver)
            .position([60.0, 300.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.slider_config("Mesh detail", 1, 4)
                    .build(&mut controls.mesh_resolution_level);
                ui.slider_config("Smoothness", 1, 6)
                    .build(&mut controls.smoothness_level);

                let y_low = controls.y_low_limit;
                let y_range_max = (383 - y_low as isize).max(2) as usize;
                ui.slider_config("Limit Y", -64, 383)
                    .build(&mut controls.y_low_limit);
                ui.slider_config("Y Range", 1, y_range_max)
                    .build(&mut controls.y_size);
                ui.separator();
                controls.apply |= ui.button_with_size("APPLY", [0.0, 0.0]);
            });
    };

    Box::new(builder)
}

fn get_statistics_menu_builder(
    state: &RenderState,
    camera: &Camera,
    world: &discrete::World,
    poly_options: PolygonizationOptions,
) -> UIWindowBuilder {
    let position = camera.get_position();
    let direction = camera.get_direction();
    let fps = state.timing.fps();
    let is_cursor_captured = state.cursor_captured;
    let chunk_position = get_minecraft_chunk_position(position);
    let block_at_position = world.get_block(position);
    let render_mode = state.render_mode;

    let density = get_density(world, position, poly_options.kernel_size);
    let gradient = get_smooth_normal(world, position, poly_options.kernel_size);

    let builder = move |ui: &imgui::Ui, _: &mut SmoothMeshOptions| {
        ui.window("stats")
            .position([60.0, 60.0], imgui::Condition::FirstUseEver)
            //.size([270.0, 120.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("fps: {:.2}", fps));
                ui.text(format!("cursor captured: {}", is_cursor_captured));
                ui.text(format!("rendering mode: {render_mode:?}"));
                ui.separator();
                ui.text(format!(
                    "position: x: {:.2} y: {:.2} z: {:.2}",
                    position.x, position.y, position.z
                ));
                ui.text(format!(
                    "direction: x: {:.2} y: {:.2} z: {:.2}",
                    direction.x, direction.y, direction.z
                ));

                ui.separator();
                ui.text(format!(
                    "region: [{}, {}]",
                    chunk_position.region_x, chunk_position.region_z
                ));
                ui.text(format!(
                    "chunk: [{}, {}]",
                    chunk_position.chunk_x, chunk_position.chunk_z
                ));
                ui.text(format!("block: {:?}", block_at_position));

                ui.separator();
                ui.text(format!("density: {}", density));
                ui.text(format!(
                    "gradient: {:.2} {:.2} {:.2}",
                    gradient.x, gradient.y, gradient.z
                ));
            });
    };

    Box::new(builder)
}

fn create_state(
    events: &Vec<InputAction>,
    old_state: RenderState,
    window: &Window,
) -> Option<RenderState> {
    let mut cursor_captured = old_state.cursor_captured;
    let mut should_render = true;
    let mut render_wireframe = old_state.render_wireframe;
    let mut render_mode = old_state.render_mode;

    for action in events {
        match action {
            InputAction::Quit => should_render = false,
            InputAction::Capture => {
                cursor_captured = !cursor_captured;
                capture_cursor(window, cursor_captured);
            }
            InputAction::KeyPressed {
                key: VirtualKeyCode::B,
            } => render_wireframe = !render_wireframe,
            InputAction::KeyPressed {
                key: VirtualKeyCode::U,
            } => render_mode = RenderingMode::Discrete,
            InputAction::KeyPressed {
                key: VirtualKeyCode::I,
            } => render_mode = RenderingMode::Implicit,
            _ => (),
        };
    }

    if !should_render {
        return None;
    }

    Some(RenderState {
        timing: old_state.timing,
        cursor_captured,
        render_wireframe,
        render_mode,
    })
}

fn capture_cursor(window: &Window, capture: bool) {
    let grab_mode = match capture {
        true => CursorGrabMode::Confined,
        false => CursorGrabMode::None,
    };

    window.set_cursor_grab(grab_mode).unwrap();
    window.set_cursor_visible(capture == false);
}

fn create_window() -> (EventLoop<()>, glium::Display) {
    let event_loop = EventLoop::new();
    let context = glium::glutin::ContextBuilder::new()
        .with_gl(glium::glutin::GlRequest::Latest)
        .with_gl_profile(glium::glutin::GlProfile::Core)
        .with_depth_buffer(24)
        .with_vsync(false);

    let builder = glium::glutin::window::WindowBuilder::new()
        .with_title(config::TITLE.to_owned())
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(1024f64, 768f64));
    let display =
        glium::Display::new(builder, context, &event_loop).expect("Failed to initialize display");

    (event_loop, display)
}

fn create_rigid_scene<'a>(
    world: &World,
    display: &Display,
) -> RenderPass<'a, model::chunk::BlockData, infrastructure::vertex::TexturedVertex, IndexBuffer<u32>>
{
    let (vertex_buffer, indices) = geometry::cube_textured_exclusive_vertex(display);
    let instance_positions = {
        let blocks = world.get_rigid_blocks_data();
        glium::vertex::VertexBuffer::new(display, &blocks).unwrap()
    };

    let cube_fragment = RenderFragmentBuilder::new()
        .set_geometry(vertex_buffer, indices)
        .set_vertex_shader(DISCRETE_VS)
        .set_fragment_shader(DISCRETE_FS)
        .build(display)
        .unwrap();

    RenderPass::new_instanced(cube_fragment, instance_positions)
}

fn create_discrete_scene<'a>(
    world: &World,
    display: &Display,
) -> RenderPass<'a, model::chunk::BlockData, infrastructure::vertex::TexturedVertex, IndexBuffer<u32>>
{
    let (vertex_buffer, indices) = geometry::cube_textured_exclusive_vertex(display);
    let instance_positions = {
        let blocks = world.get_surface_block_data();
        glium::vertex::VertexBuffer::new(display, &blocks).unwrap()
    };

    let cube_fragment = RenderFragmentBuilder::new()
        .set_geometry(vertex_buffer, indices)
        .set_vertex_shader(DISCRETE_VS)
        .set_fragment_shader(DISCRETE_FS)
        .build(display)
        .unwrap();

    RenderPass::new_instanced(cube_fragment, instance_positions)
}

fn create_implicit_scene<'a>(
    world: &World,
    display: &Display,
) -> RenderPass<'a, NoInstance, MeshVertex, IndexBuffer<u32>> {
    let smooth_mesh = world.get_smooth_mesh();

    let vertex_buffer = glium::VertexBuffer::new(display, &smooth_mesh.vertices).unwrap();
    let index_buffer = glium::IndexBuffer::new(
        display,
        glium::index::PrimitiveType::TrianglesList,
        &smooth_mesh.indices,
    )
    .unwrap();

    let fragment = RenderFragmentBuilder::new()
        .set_geometry(vertex_buffer, index_buffer)
        .set_vertex_shader(IMPLICIT_VS)
        .set_fragment_shader(IMPLICIT_FS)
        .build(display)
        .unwrap();

    RenderPass::new(fragment)
}

fn create_camera(window_dimensions: (u32, u32)) -> Camera {
    let aspect_ratio = window_dimensions.0 as Real / window_dimensions.1 as Real;

    Camera::new(
        config::SPAWN_POINT,
        config::SPAWN_DIR,
        Vector3::unit_y(),
        config::FOVY,
        aspect_ratio,
        config::Z_NEAR,
        config::Z_FAR,
    )
}
