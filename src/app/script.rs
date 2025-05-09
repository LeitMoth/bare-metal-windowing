use pluggable_interrupt_os::vga_buffer::{Color, ColorCode};
use simple_interp::{Interpreter, InterpreterOutput};

use crate::InterpType;

use super::window::Window;

#[derive(Debug)]
pub struct RunningScript {
    window: Window,
    interpreter: InterpType,
}

impl RunningScript {
    pub fn new(window: Window, interpreter: InterpType) -> Self {
        Self {
            window,
            interpreter,
        }
    }

    pub fn draw(&self) {
        self.window
            .plot('H', 0, 0, ColorCode::new(Color::LightRed, Color::Black));
    }
}

impl InterpreterOutput for RunningScript {
    fn print(&mut self, chars: &[u8]) {
        // self.window.plot(c, col, row, color);
    }
}
