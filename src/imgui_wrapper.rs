use glium::glutin::event::Event;
use glium::glutin::window::Window;
use glium::Frame;
use imgui::Ui;
use std::time::Duration;

// context + platform + renderer
// methods: prepare, render
// can be behind a trait - Renderable/RenderFragment/SceneFragment
// main has draw_scene: clear + call renderables + finish


pub struct ImguiWrapper {
    context: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_glium_renderer::Renderer,
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
        }
    }

    pub fn prepare(&mut self, window: &Window, delta_time: Duration) {
        self.context.io_mut().update_delta_time(delta_time);

        self.platform
            .prepare_frame(self.context.io_mut(), window)
            .expect("Failed to prepare frame");
    }

    pub fn render<F: FnOnce(&Ui)>(&mut self, window: &Window, target: &mut Frame, builder: F) {
        let ui = self.context.new_frame();

        builder(ui);

        self.platform.prepare_render(ui, window);
        let draw_data = self.context.render();
        self.renderer
            .render(target, draw_data)
            .expect("Rendering failed");

    }

    pub fn handle_event<T>(&mut self, window: &Window, event: &Event<T>) {
        self.platform
            .handle_event(self.context.io_mut(), window, event);
    }
}
