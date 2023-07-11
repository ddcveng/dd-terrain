use glium::glutin::event::Event;
use glium::glutin::window::Window;
use glium::Frame;
use imgui_glium_renderer::RendererError;
use std::time::Duration;

#[derive(Clone, Copy)]
pub struct SmoothMeshOptions {
    pub smoothness_level: u8,
    pub mesh_resolution_level: u8,
    pub y_low_limit: isize,
    pub y_size: usize,
    pub apply: bool,
}

impl Default for SmoothMeshOptions {
    fn default() -> Self {
        SmoothMeshOptions {
            smoothness_level: 2,
            mesh_resolution_level: 1,
            y_low_limit: 40,
            y_size: 40,
            apply: false,
        }
    }
}

pub type UIWindowBuilder = Box<dyn FnOnce(&imgui::Ui, &mut SmoothMeshOptions)>;

pub struct ImguiWrapper {
    context: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_glium_renderer::Renderer,
    window_builders: Vec<UIWindowBuilder>,
}

impl ImguiWrapper {
    pub fn new(display: &glium::Display) -> Self {
        let mut imgui_context = imgui::Context::create();
        imgui_context.set_ini_filename(None);

        let mut winit_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);

        let gl_window = display.gl_window();
        let window = gl_window.window();

        let dpi_mode = imgui_winit_support::HiDpiMode::Default;

        winit_platform.attach_window(imgui_context.io_mut(), window, dpi_mode);

        imgui_context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

        let imgui_renderer = imgui_glium_renderer::Renderer::init(&mut imgui_context, display)
            .expect("failed to init imgui renderer");

        ImguiWrapper {
            context: imgui_context,
            platform: winit_platform,
            renderer: imgui_renderer,
            window_builders: Vec::new(),
        }
    }

    pub fn prepare(&mut self, window: &Window, delta_time: Duration) {
        self.context.io_mut().update_delta_time(delta_time);

        self.platform
            .prepare_frame(self.context.io_mut(), window)
            .expect("Failed to prepare frame");
    }

    pub fn add_window(&mut self, window_builder: UIWindowBuilder) {
        self.window_builders.push(window_builder);
    }

    pub fn render_frame(
        &mut self,
        window: &Window,
        target: &mut Frame,
        controls: &mut SmoothMeshOptions,
    ) -> Result<(), RendererError> {
        let ui = self.context.new_frame();

        for builder in self.window_builders.drain(..) {
            builder(ui, controls);
        }

        self.platform.prepare_render(ui, window);
        let draw_data = self.context.render();

        self.renderer.render(target, draw_data)
    }

    pub fn handle_event<T>(&mut self, window: &Window, event: &Event<T>) {
        self.platform
            .handle_event(self.context.io_mut(), window, event);
    }
}
