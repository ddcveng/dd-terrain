use std::time::Duration;

use glium::backend::Facade;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::{uniform, Surface, implement_vertex};

mod imgui_wrapper;
use glutin::event::VirtualKeyCode;
use imgui_wrapper::ImguiWrapper;

mod minecraft;
mod camera;
use camera::Camera;
mod geometry;
mod infrastructure;
use infrastructure::input::{InputAction, self, InputConsumer};
use infrastructure::render_fragment::RenderFragmentBuilder;
use infrastructure::{RenderState, Timing};
use cgmath::{Rad, Point3, Vector3, EuclideanSpace};

const TITLE: &str = "dd-terrain";
const VS_SOURCE: &str = include_str!("shaders/vs.glsl");
const FS_SOURCE: &str = include_str!("shaders/fs.glsl");
const FOVY: Rad<f32> = Rad(std::f32::consts::FRAC_PI_2);
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 10.;

fn main() {
    let (event_loop, display) = create_window();

    let (vertex_buffer, indices) = geometry::cube_color_exclusive_vertex(&display);
    let instance_positions = {
        let blocks = minecraft::get_chunk();
        glium::vertex::VertexBuffer::new(&display, &blocks).unwrap()
    };

    let triangle_renderer = RenderFragmentBuilder::new()
        .set_geometry(vertex_buffer, indices)
        .set_vertex_shader(VS_SOURCE)
        .set_fragment_shader(FS_SOURCE)
        .build(&display)
        .unwrap();

    let mut imgui_data = ImguiWrapper::new(&display);
    let dimensions = display.get_framebuffer_dimensions();
    let aspect_ratio = dimensions.0 / dimensions.1;
    let mut camera = Camera::new(
        Point3::new(-2., 0., 4.),
        Point3::origin(),
        Vector3::unit_y(),
        FOVY,
        aspect_ratio as f32,
        Z_NEAR,
        Z_FAR,
    );

    // Timer for FPS calculation
    let mut timings = Timing::new();
    let mut render_state = RenderState {
        timing: Timing::new(),
        cursor_captured: false,
        should_render: true,
        render_wireframe: false,
    };
    let mut actions: Vec<InputAction> = Vec::new();

    // Standard winit event loop
    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(_) => {
            actions.clear();
            timings.record_frame();
        }
        Event::MainEventsCleared => {
            let gl_window = display.gl_window();
            render_state = create_state(&actions, render_state, timings);
            if !render_state.should_render {
                *control_flow = ControlFlow::Exit;
                return;
            }

            imgui_data.prepare(gl_window.window(), render_state.timing.delta_time);

            for action in &actions {
                camera.consume(action, &render_state);
            }

            camera.update(render_state.timing.delta_time.as_secs_f32());

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
            let model: [[f32; 4]; 4] = cgmath::Matrix4::from_scale(0.3).into();
            let projection: [[f32; 4]; 4] = camera.projection.into();
            let view: [[f32; 4]; 4] = camera.world_to_view.into();
            let uni = uniform! {
                projection: projection,
                view: view,
                model: model,
            };

            triangle_renderer.render_instanced(&mut target, &uni, &instance_positions, render_state.render_wireframe);

            // Draw imgui last so it shows on top of everything
            let imgui_frame_builder = get_imgui_builder(&render_state, &camera);
            imgui_data.render(gl_window.window(), &mut target, imgui_frame_builder);
            //imgui_data.render(gl_window.window(), &mut target);

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

fn get_imgui_builder(state: &RenderState, camera: &Camera) -> impl FnOnce(&imgui::Ui) {
    let position = camera.get_position();
    let direction = camera.get_direction();
    let fps = state.timing.fps();
    let is_cursor_captured = state.cursor_captured;

    let builder = move |ui: &imgui::Ui| {
        ui.window("stats")
            //.size([270.0, 120.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("fps: {:.2}", fps));
                ui.text(format!("cursor captured: {}", is_cursor_captured));
                ui.separator();
                ui.text(format!("position: x: {:.2} y: {:.2} z: {:.2}", position.x, position.y, position.z));
                ui.text(format!("direction: x: {:.2} y: {:.2} z: {:.2}", direction.x, direction.y, direction.z));
            });
    };

    builder
}

fn create_state(events: &Vec<InputAction>, old_state: RenderState, timing: Timing) -> RenderState {
    let mut cursor_captured = old_state.cursor_captured;
    let mut should_render = true;
    let mut render_wireframe = old_state.render_wireframe;

    for action in events {
        match action {
            InputAction::Quit => should_render = false,
            InputAction::Capture => cursor_captured = !cursor_captured, // TODO: hide cursor
            InputAction::KeyPressed { key: VirtualKeyCode::B } => render_wireframe = !render_wireframe,
            _ => (),
        };
    }

    RenderState {
        timing,
        cursor_captured,
        should_render,
        render_wireframe,
    }
}

fn create_window() -> (EventLoop<()>, glium::Display) {
    let event_loop = EventLoop::new();
    let context = glium::glutin::ContextBuilder::new()
        .with_gl(glium::glutin::GlRequest::Latest)
        .with_gl_profile(glium::glutin::GlProfile::Core)
        .with_depth_buffer(24)
        .with_vsync(true);

    let builder = glium::glutin::window::WindowBuilder::new()
        .with_title(TITLE.to_owned())
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(1024f64, 768f64));
    let display =
        glium::Display::new(builder, context, &event_loop).expect("Failed to initialize display");

    (event_loop, display)
}
