mod renderer;
use renderer::Renderer;
mod wavefront_object;
use std::process::exit;
use wavefront_object::Obj;

mod vertex;

mod camera;
use camera::Camera;

mod input;
use input::InputAction;
use input::InputConsumer;

//mod imgui_renderer;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 540;

//const VERTICES: [Vertex2D; 3] = [
//    Vertex2D::new(
//        VertexPosition::new([-0.5, -0.5]),
//        VertexRGB::new([255, 0, 0]),
//    ),
//    Vertex2D::new(
//        VertexPosition::new([0.5, -0.5]),
//        VertexRGB::new([0, 255, 0]),
//    ),
//    Vertex2D::new(VertexPosition::new([0., 0.5]), VertexRGB::new([0, 0, 255])),
//];

fn main() {
    let renderer = Renderer::new("banan", WIDTH, HEIGHT);

    match renderer {
        Ok(mut renderer) => {
            eprintln!("Graphics surface created");
            renderer.main_loop();
        }

        Err(e) => {
            eprintln!("cannot create graphics surface:\n{:?}", e);
            exit(1);
        }
    }
}

//fn main_loop(surface: GlfwSurface) {
//    let start_t = Instant::now();
//    let mut prev_t = start_t.clone();
//    let mut ctxt = surface.context;
//    let event_rx = surface.events_rx;
//    let back_buffer = ctxt.back_buffer().expect("back buffer");
//
//    let suzanne_path = std::path::Path::new("./suzanne.obj");
//    let suzanne = Obj::load(suzanne_path).unwrap();
//    let suzanne_tess = suzanne.to_tess(&mut ctxt).unwrap();
//    //    let triange = ctxt
//    //        .new_tess()
//    //        .set_vertices(&VERTICES[..])
//    //        .set_mode(Mode::Triangle)
//    //        .build()
//    //        .unwrap();
//
//    let mut program = ctxt
//        .new_shader_program::<VertexSemantics, (), ShaderInterface>()
//        .from_strings(VS_STR, None, None, FS_STR)
//        .unwrap()
//        .ignore_warnings();
//
//    let mut camera = Camera::new(
//        Point3::new(2., 2., 2.),
//        Point3::origin(),
//        Vector3::unit_y(),
//        FOVY,
//        WIDTH as f32 / HEIGHT as f32,
//        Z_NEAR,
//        Z_FAR,
//    );
//
//    unsafe {
//        glfw::ffi::glfwSetInputMode(
//            ctxt.window.window_ptr(),
//            glfw::ffi::CURSOR,
//            glfw::ffi::CURSOR_DISABLED,
//        );
//    }
//
//    'app: loop {
//        let time = Instant::now();
//        let delta_time = time.duration_since(prev_t).as_secs_f32();
//        prev_t = time;
//
//        ctxt.window.glfw.poll_events();
//        let events =
//            glfw::flush_messages(&event_rx).flat_map(|(_, event)| input::translate_event(event));
//        for event in events {
//            match event {
//                InputAction::Quit => break 'app,
//                _ => (),
//            }
//
//            camera.consume(&event, delta_time);
//        }
//
//        // Rendering code
//
//        let t = start_t.elapsed().as_secs_f32();
//        let color = [t.cos(), t.sin(), 0.5, 1.];
//        //        let projection = perspective(FOVY, WIDTH as f32 / HEIGHT as f32, Z_NEAR, Z_FAR);
//        //
//        //        let radius = 5.0;
//        //        let cam_x = t.cos() * radius;
//        //        let cam_y = t.sin() * radius;
//        //        let view = Matrix4::<f32>::look_at_rh(
//        //            Point3::new(cam_x, 2., cam_y),
//        //            Point3::origin(),
//        //            Vector3::unit_y(),
//        //        );
//
//        let render = ctxt
//            .new_pipeline_gate()
//            .pipeline(
//                &back_buffer,
//                &PipelineState::default().set_clear_color(color),
//                |_pipeline, mut shading_gate| {
//                    shading_gate.shade(&mut program, |mut iface, uniforms, mut render_gate| {
//                        // TODO: create a macro/trait for conversion of cgmath Matrix4 and luminance
//                        // Mat44. Or implement Uniformable for Matrix4
//                        let projection_arr: [[f32; 4]; 4] = camera.projection.into();
//                        iface.set(&uniforms.projection, projection_arr.into());
//
//                        let view_arr: [[f32; 4]; 4] = camera.view.into();
//                        iface.set(&uniforms.view, view_arr.into());
//
//                        render_gate.render(&RenderState::default(), |mut tesselation_gate| {
//                            tesselation_gate.render(&suzanne_tess)
//                        })
//                    })
//                },
//            )
//            .assume();
//
//        if render.is_ok() {
//            ctxt.window.swap_buffers();
//        } else {
//            break 'app;
//        }
//    }
//}
