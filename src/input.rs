use glfw::{Action, Key, MouseButton, WindowEvent};

#[derive(Debug)]
pub enum Direction {
    Forward,
    Back,
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug)]
pub enum InputAction {
    Quit,
    BeginMove { dir: Direction },
    EndMove { dir: Direction },
    CursorMoved { x: f64, y: f64 },
    Scroll(f64, f64),
    MousePressed { button: MouseButton },
    KeyPressed { key: Key },
    Char { c: char },
    Capture,
}

pub fn translate_event(event: WindowEvent) -> Option<InputAction> {
    match event {
        WindowEvent::Close | WindowEvent::Key(Key::Q, _, Action::Release, _) => {
            Some(InputAction::Quit)
        }
        WindowEvent::Key(key, _, Action::Press, _) => match key {
            Key::W => Some(InputAction::BeginMove {
                dir: Direction::Forward,
            }),
            Key::A => Some(InputAction::BeginMove {
                dir: Direction::Left,
            }),
            Key::S => Some(InputAction::BeginMove {
                dir: Direction::Back,
            }),
            Key::D => Some(InputAction::BeginMove {
                dir: Direction::Right,
            }),
            Key::K => Some(InputAction::BeginMove { dir: Direction::Up }),
            Key::J => Some(InputAction::BeginMove {
                dir: Direction::Down,
            }),
            Key::Space => Some(InputAction::Capture),
            _ => Some(InputAction::KeyPressed { key }),
        },
        WindowEvent::Key(key, _, Action::Release, _) => match key {
            Key::W => Some(InputAction::EndMove {
                dir: Direction::Forward,
            }),
            Key::A => Some(InputAction::EndMove {
                dir: Direction::Left,
            }),
            Key::S => Some(InputAction::EndMove {
                dir: Direction::Back,
            }),
            Key::D => Some(InputAction::EndMove {
                dir: Direction::Right,
            }),
            Key::K => Some(InputAction::EndMove { dir: Direction::Up }),
            Key::J => Some(InputAction::EndMove {
                dir: Direction::Down,
            }),
            _ => None,
        },
        WindowEvent::Char(c) => Some(InputAction::Char { c }),
        WindowEvent::CursorPos(x, y) => Some(InputAction::CursorMoved { x, y }),
        WindowEvent::Scroll(x, y) => Some(InputAction::Scroll(x, y)),
        WindowEvent::MouseButton(button, Action::Press, _) => {
            Some(InputAction::MousePressed { button })
        }
        _ => None,
    }
}

pub trait InputConsumer {
    fn consume(&mut self, action: &InputAction, delta_t: f32, cursor_captured: bool) -> ();
}
