use std::time::{Duration, Instant};

#[derive(Debug, Copy, Clone)]
pub struct RenderState {
    pub timing: Timing,
    pub cursor_captured: bool,
    pub render_wireframe: bool,
}

impl RenderState {
    pub fn new() -> Self {
        RenderState { 
            timing: Timing::new(),
            cursor_captured: false,
            render_wireframe: false 
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Timing {
    pub delta_time: Duration,
    pub running_time: Duration,
    starting_time: Instant,
    last_frame: Instant,
}

impl Timing {
    pub fn new() -> Self {
        let now = Instant::now();
        Timing { 
            delta_time: Duration::ZERO,
            running_time: Duration::ZERO,
            starting_time: now,
            last_frame: now,
        }
    }

    pub fn record_frame(&mut self) {
        let now = Instant::now();
        self.delta_time = now - self.last_frame;
        self.running_time = now - self.starting_time;
        self.last_frame = now;
    }

    pub fn fps(&self) -> f32 {
        1.0 / self.delta_time.as_secs_f32()
    }
}
