use core::any::Any;

use core::fmt::Write;
use pluggable_interrupt_os::{
    println,
    vga_buffer::{plot, Color, ColorCode},
};
use simple_interp::{ArrayString, Interpreter, InterpreterOutput, TickStatus};

use crate::InterpType;

use super::window::Window;

pub struct RunningScript {
    window: Window,
    interpreter: InterpType,
    iobuffer: IOBuffer,
    status: TickStatus,
}

#[derive(Default)]
struct IOBuffer {
    buf: ArrayString<1024>,
    input_pos: Option<usize>,
}

impl IOBuffer {
    fn begin_input(&mut self) {
        self.input_pos = Some(self.buf.len())
    }

    fn end_input(&mut self) -> &[u8] {
        match self.input_pos {
            None => &[],
            Some(begin) => &self.buf.buffer_slice()[begin..self.buf.buffer_slice().len() - 1],
        }
    }
}

impl InterpreterOutput for IOBuffer {
    fn print(&mut self, chars: &[u8]) {
        for c in chars {
            self.buf.push_char(*c as char);
        }
    }
}

impl RunningScript {
    pub fn new(window: Window, interpreter: InterpType) -> Self {
        let outbuffer = Default::default();
        window.clear();
        Self {
            window,
            interpreter,
            iobuffer: outbuffer,
            status: TickStatus::Continuing,
        }
    }

    pub fn tick(&mut self) -> bool {
        match self.status {
            TickStatus::Continuing => {
                self.status = self.interpreter.tick(&mut self.iobuffer);
                if let TickStatus::AwaitInput = self.status {
                    self.iobuffer.begin_input();
                }
                true
            }
            TickStatus::Finished => false,
            TickStatus::AwaitInput => {
                if let Some(c) = self.iobuffer.buf.buffer_slice().last() {
                    let c = *c as char;
                    if c == '\n' {
                        let input = self.iobuffer.end_input();
                        let input = str::from_utf8(input).unwrap_or("");
                        match self.interpreter.provide_input(input) {
                            Ok(()) => (),
                            Err(e) => write!(self.iobuffer.buf, "{e}").unwrap_or(()),
                        }
                    }
                }

                false
            }
        }
    }

    pub fn draw(&mut self) {
        let color = ColorCode::new(Color::LightGray, Color::Black);

        let mut cursor = 0;

        let pplot = |c, cursor| {
            self.window.plot(
                c,
                (cursor % self.window.width()) as u8,
                (cursor / self.window.width()) as u8,
                color,
            );
        };
        for c in self.iobuffer.buf.buffer_slice() {
            let c = *c as char;
            if c == '\n' {
                let tmp = cursor + self.window.width();
                let tmp = tmp - tmp % self.window.width();
                for _ in cursor..tmp {
                    pplot(' ', cursor);
                    cursor += 1;
                }
            } else {
                pplot(c, cursor);
                cursor += 1;
            }
        }
    }
}
