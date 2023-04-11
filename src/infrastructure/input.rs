use glium::glutin::event::{Event, VirtualKeyCode, WindowEvent};
use glutin::event::{DeviceEvent, ElementState};

use crate::RenderState;

#[derive(Debug)]
pub enum Direction {
    Forward,
    Back,
    Left,
    Right,
    Up,
    Down,
}

type Key = VirtualKeyCode;
type MouseButton = glutin::event::MouseButton;

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
    Resized(u32, u32),
}

pub fn translate_event(event: Event<()>) -> Option<InputAction> {
    match event {
        Event::WindowEvent {
            event: window_event,
            ..
        } => translate_window_event(window_event),
        Event::DeviceEvent {
            event: device_event,
            ..
        } => translate_device_event(device_event),
        _ => None,
    }
}

fn translate_window_event(event: WindowEvent) -> Option<InputAction> {
    match event {
        WindowEvent::CloseRequested => Some(InputAction::Quit),
        WindowEvent::KeyboardInput {
            device_id: _,
            input,
            is_synthetic: false,
        } => handle_keypress(&input),
        WindowEvent::ReceivedCharacter(c) => Some(InputAction::Char { c }),
        WindowEvent::Resized(size) => Some(InputAction::Resized(size.width, size.height)),
        _ => None,
    }
}

fn translate_device_event(event: DeviceEvent) -> Option<InputAction> {
    match event {
        DeviceEvent::MouseMotion { delta } => Some(InputAction::CursorMoved {
            x: delta.0,
            y: delta.1,
        }),
        _ => None,
    }
}

fn handle_keypress(event: &glutin::event::KeyboardInput) -> Option<InputAction> {
    let pressed = event.state == ElementState::Pressed;
    let keycode = match event.virtual_keycode {
        Some(key) => key,
        None => return None,
    };

    let move_action = |dir: Direction| -> Option<InputAction> {
        if pressed {
            Some(InputAction::BeginMove { dir })
        } else {
            Some(InputAction::EndMove { dir })
        }
    };

    let if_pressed = |action: InputAction| -> Option<InputAction> {
        if pressed {
            Some(action)
        } else {
            None
        }
    };

    match keycode {
        VirtualKeyCode::W => move_action(Direction::Forward),
        VirtualKeyCode::A => move_action(Direction::Left),
        VirtualKeyCode::S => move_action(Direction::Back),
        VirtualKeyCode::D => move_action(Direction::Right),
        VirtualKeyCode::J => move_action(Direction::Down),
        VirtualKeyCode::K => move_action(Direction::Up),
        VirtualKeyCode::Space => if_pressed(InputAction::Capture),
        VirtualKeyCode::Q => Some(InputAction::Quit),
        _ => if_pressed(InputAction::KeyPressed { key: keycode }),
    }
}

pub trait InputConsumer {
    fn consume(&mut self, action: &InputAction, state: &RenderState) -> ();
}
