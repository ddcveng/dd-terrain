use cgmath::{EuclideanSpace, Point3, Rad, Vector3};
use glfw::{Context, SwapInterval, WindowEvent, WindowMode};
use luminance::context::GraphicsContext;
use luminance::pipeline::PipelineState;
use luminance::render_state::RenderState;
use luminance::shader::types::Mat44;
use luminance::shader::Uniform;
use luminance_derive::UniformInterface;
use luminance_glfw::{GL33Context, GlfwSurface, GlfwSurfaceError};
use std::fmt::Debug;
use std::sync::mpsc::Receiver;

use crate::input;
use crate::vertex::VertexSemantics;
use crate::{Camera, InputAction, InputConsumer, Obj};

const VS_STR: &str = include_str!("shaders/vs.glsl");
const FS_STR: &str = include_str!("shaders/fs.glsl");
const FOVY: Rad<f32> = Rad(std::f32::consts::FRAC_PI_2);
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 10.;

#[derive(Debug)]
pub enum PlatformError {
    CannotCreateWindow,
}

// TODO: make cgmath::Matrix4 uniformable
#[derive(Debug, UniformInterface)]
struct ShaderInterface {
    #[uniform(unbound)]
    projection: Uniform<Mat44<f32>>,

    #[uniform(unbound)]
    view: Uniform<Mat44<f32>>,
}

struct WindowOptions {
    width: u32,
    height: u32,
}

pub struct Renderer {
    context: GL33Context,
    events_rx: Receiver<(f64, WindowEvent)>,
    window_options: WindowOptions,
}

impl Renderer {
    pub fn new(
        window_name: &str,
        window_width: u32,
        window_height: u32,
    ) -> Result<Self, GlfwSurfaceError<PlatformError>> {
        let surface = GlfwSurface::new(|glfw| {
            let (mut window, events) = glfw
                .create_window(
                    window_width,
                    window_height,
                    window_name,
                    WindowMode::Windowed,
                )
                .ok_or_else(|| GlfwSurfaceError::UserError(PlatformError::CannotCreateWindow))?;

            window.make_current();
            window.set_all_polling(true);
            glfw.set_swap_interval(SwapInterval::Sync(1));

            Ok((window, events))
        });

        match surface {
            Ok(surface) => Ok(Renderer {
                context: surface.context,
                events_rx: surface.events_rx,
                //running: false,
                window_options: WindowOptions {
                    width: window_width,
                    height: window_height,
                },
                //cursor_captured: false,
            }),
            Err(e) => Err(e),
        }
    }

    pub fn main_loop(&mut self) {
        let mut prev_t = 0.0;
        let back_buffer = self.context.back_buffer().expect("back buffer");

        let suzanne_path = std::path::Path::new("./suzanne.obj");
        let suzanne = Obj::load(suzanne_path).unwrap();
        let suzanne_tess = suzanne.to_tess(&mut self.context).unwrap();

        let mut program = self
            .context
            .new_shader_program::<VertexSemantics, (), ShaderInterface>()
            .from_strings(VS_STR, None, None, FS_STR)
            .unwrap()
            .ignore_warnings();

        let aspect_ratio = self.window_options.width as f32 / self.window_options.height as f32;
        let mut camera = Camera::new(
            Point3::new(2., 2., 2.),
            Point3::origin(),
            Vector3::unit_y(),
            FOVY,
            aspect_ratio,
            Z_NEAR,
            Z_FAR,
        );

        //        unsafe {
        //            glfw::ffi::glfwSetInputMode(
        //                self.context.window.window_ptr(),
        //                glfw::ffi::CURSOR,
        //                glfw::ffi::CURSOR_DISABLED,
        //            );
        //        }

        let mut cursor_captured = false;
        'app: loop {
            // Calculate time between frames
            let time = self.context.window.glfw.get_time() as f32;
            let delta_time = time - prev_t;
            prev_t = time;

            // Process events
            self.context.window.glfw.poll_events();
            let events = glfw::flush_messages(&self.events_rx)
                .flat_map(|(_, event)| input::translate_event(event));
            for event in events {
                match event {
                    InputAction::Quit => break 'app,
                    InputAction::Capture => {
                        cursor_captured = !cursor_captured;
                        unsafe {
                            capture_mouse(self.context.window.window_ptr(), cursor_captured);
                        }
                    }
                    _ => (),
                }

                camera.consume(&event, delta_time, cursor_captured);
            }

            camera.update(delta_time);

            // Rendering code
            let color = [time.cos(), time.sin(), 0.5, 1.];
            let render = self
                .context
                .new_pipeline_gate()
                .pipeline(
                    &back_buffer,
                    &PipelineState::default().set_clear_color(color),
                    |_pipeline, mut shading_gate| {
                        shading_gate.shade(&mut program, |mut iface, uniforms, mut render_gate| {
                            // TODO: create a macro/trait for conversion of cgmath Matrix4 and luminance
                            // Mat44. Or implement Uniformable for Matrix4
                            let projection_arr: [[f32; 4]; 4] = camera.projection.into();
                            iface.set(&uniforms.projection, projection_arr.into());

                            let view_arr: [[f32; 4]; 4] = camera.view.into();
                            iface.set(&uniforms.view, view_arr.into());

                            render_gate.render(&RenderState::default(), |mut tesselation_gate| {
                                tesselation_gate.render(&suzanne_tess)
                            })
                        })
                    },
                )
                .assume();

            if render.is_ok() {
                self.context.window.swap_buffers();
            } else {
                break 'app;
            }
        }
    }
}

unsafe fn capture_mouse(window: *mut glfw::ffi::GLFWwindow, capture: bool) {
    let value = if capture {
        glfw::ffi::CURSOR_DISABLED
    } else {
        glfw::ffi::CURSOR_NORMAL
    };

    glfw::ffi::glfwSetInputMode(window, glfw::ffi::CURSOR, value);
}
