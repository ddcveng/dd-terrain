use glfw::{Action, Context as _, Key, WindowEvent, WindowMode, SwapInterval};
use luminance_glfw::{GlfwSurface, GlfwSurfaceError};
use std::process::exit;
use luminance::context::GraphicsContext as _;
use luminance::pipeline::PipelineState;
use std::time::Instant;

#[derive(Debug)]
pub enum PlatformError {
  CannotCreateWindow,
}

fn main() {
    let name = "banan";
    let surface = GlfwSurface::new(|glfw| {
        let (mut window, events) = glfw
          .create_window(960, 540, name, WindowMode::Windowed)
          .ok_or_else(|| GlfwSurfaceError::UserError(PlatformError::CannotCreateWindow))?;

        window.make_current();
        window.set_all_polling(true);
        glfw.set_swap_interval(SwapInterval::Sync(1));

        Ok((window, events))
    });

//    let surface = GlfwSurface::new(|glfw| {
//        let window = glfw.create_window(300, 300, "banan", glfw::WindowMode::Windowed);
//
//        match window {
//            None => {
//                Err(GlfwSurfaceError::<String>::InitError(glfw::InitError::Internal))
//            }
//
//            Some(win) => {
//                Ok(win)
//            }
//        }
//
//    });
    
    match surface {
        Ok(surface) => {
            eprintln!("Graphics surface created");
            main_loop(surface);
        }

        Err(e) => {
            eprintln!("cannot create graphics surface:\n{:?}", e);
            exit(1);
        }
    }
}

fn main_loop(surface: GlfwSurface) {
    let start_t = Instant::now();
    let mut ctxt = surface.context;
    let events = surface.events_rx;
    let back_buffer = ctxt.back_buffer().expect("back buffer");

    'app: loop {
        ctxt.window.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            //println!("{:?}", event);
            match event {
                WindowEvent::Close | WindowEvent::Key(Key::Q, _, Action::Release, _) => break 'app,
                _ => ()
            }
        }

        // Rendering code
        let t = start_t.elapsed().as_secs_f32();
        let color = [t.cos(), t.sin(), 0.5, 1.];

        let render = ctxt
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default().set_clear_color(color),
                |_, _| Ok(()),
            )
            .assume();

        if render.is_ok() {
            ctxt.window.swap_buffers();
        } else {
            break 'app;
        }
    }

}
